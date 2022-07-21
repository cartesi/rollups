use crate::FoldableError;
use anyhow::Context;
use async_trait::async_trait;
use contracts::rollups_facet::*;
use contracts::validator_manager_facet::*;
use ethers::{
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

pub const MAX_NUM_VALIDATORS: usize = 8;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumClaims {
    pub validator_address: Address,
    pub num_claims_made: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorManagerState {
    // each tuple containing (validator_address, #claims_made_so_far)
    // note that when a validator gets removed, the corresponding option
    // becomes `None` and this `None` can appear anywhere in the array
    pub num_claims: [Option<NumClaims>; MAX_NUM_VALIDATORS],
    // validators that have claimed in the current unfinalized epoch
    pub claiming: Vec<Address>,
    // validators that lost the disputes
    pub validators_removed: Vec<Address>,
    pub num_finalized_epochs: U256,
    pub dapp_contract_address: Address,
}

impl ValidatorManagerState {
    pub fn num_claims_for_validator(&self, validator_address: Address) -> U256 {
        // number of total claims for the validator
        let num_claims = self.num_claims;
        let mut validator_claims = U256::zero();
        for i in 0..MAX_NUM_VALIDATORS {
            // find validator address in `num_claims`
            if let Some(num_claims_struct) = &num_claims[i] {
                if num_claims_struct.validator_address == validator_address {
                    validator_claims = num_claims_struct.num_claims_made;
                    break;
                }
            }
        }
        validator_claims
    }
}

#[async_trait]
impl Foldable for ValidatorManagerState {
    type InitialState = Address;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = *initial_state;
        let middleware = access.get_inner();
        let validator_manager_facet = ValidatorManagerFacet::new(
            dapp_contract_address,
            Arc::clone(&middleware),
        );
        let rollups_facet =
            RollupsFacet::new(dapp_contract_address, Arc::clone(&middleware));

        // declare variables
        let mut num_claims: [Option<NumClaims>; 8] = [None; 8];
        let mut validators_removed: Vec<Address> = Vec::new();

        // validators that have claimed in the current unfinalized epoch
        let mut claiming: Vec<Address> = Vec::new();

        // retrive events
        // RollupsFacet ResolveDispute event
        let resolve_dispute_events = rollups_facet
            .resolve_dispute_filter()
            .query()
            .await
            .context("Error querying for resolve dispute events")?;

        // NewEpoch event
        let new_epoch_events = validator_manager_facet
            .new_epoch_filter()
            .query()
            .await
            .context("Error querying for new epoch events")?;

        // RollupsFacet Claim event
        let rollups_claim_events =
            rollups_facet
                .claim_filter()
                .query()
                .await
                .context("Error querying for Rollups claim events")?;

        // step 1: `resolve_dispute_events`. For validator lost dispute, add to removal list; for validator won, do nothing
        // step 2: for every finalized epoch, if a validator made a claim, and its address has not been removed, then #claims++.
        //          Those who made a false claim have been removed in step 1 already.
        // step 3: for epoch that hasn't been finalized (no more than 1 such epoch), store which honest validators have claimed
        //          No need to store the claim content. Because the dishonest will be removed before epoch finalized

        // step 1:
        for ev in resolve_dispute_events.iter() {
            let losing_validator = ev.loser;
            validators_removed.push(losing_validator);
        }

        let num_finalized_epochs = U256::from(new_epoch_events.len());
        for ev in rollups_claim_events.iter() {
            let claim_epoch_num = ev.epoch_number;
            let claimer = ev.claimer;
            if validators_removed.contains(&claimer) {
                continue;
            }
            if claim_epoch_num < num_finalized_epochs {
                // step 2
                for i in 0..MAX_NUM_VALIDATORS {
                    // find claimer in `num_claims`
                    if let Some(num_claims_struct) = &num_claims[i] {
                        let addr = num_claims_struct.validator_address;
                        let num = num_claims_struct.num_claims_made;
                        if addr == claimer {
                            num_claims[i] = Some(NumClaims {
                                validator_address: addr,
                                num_claims_made: num + 1,
                            });
                            break;
                        } else {
                            continue;
                        }
                    }
                    if let None = num_claims[i] {
                        // at this stage, there's no `None` between `Some`
                        num_claims[i] = Some(NumClaims {
                            validator_address: claimer,
                            num_claims_made: U256::one(),
                        });
                        break;
                    }
                }
            } else {
                // step 3
                claiming.push(ev.claimer);
            }
        }

        Ok(ValidatorManagerState {
            num_claims,
            claiming,
            validators_removed,
            num_finalized_epochs,
            dapp_contract_address,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = previous_state.dapp_contract_address;
        // the following logic is: if `dapp_contract_address` and any of the Validator Manager
        // Facet events are in the bloom, or if `dapp_contract_address` and `ClaimFilter` event are in the bloom,
        // then skip this `if` statement and do the logic below.
        // Otherwise, return the previous state
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &DisputeEndedFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &NewEpochFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimFilter::signature(),
        ))) {
            return Ok(previous_state.clone());
        }

        let middleware = env.inner_middleware();
        let validator_manager_facet = ValidatorManagerFacet::new(
            dapp_contract_address,
            Arc::clone(&middleware),
        );
        let rollups_facet =
            RollupsFacet::new(dapp_contract_address, Arc::clone(&middleware));
        let mut state = previous_state.clone();

        // retrive events

        // RollupsFacet ResolveDispute event
        let resolve_dispute_events = rollups_facet
            .resolve_dispute_filter()
            .query()
            .await
            .context("Error querying for resolve dispute events")?;

        // NewEpoch event
        let new_epoch_events = validator_manager_facet
            .new_epoch_filter()
            .query()
            .await
            .context("Error querying for new epoch events")?;

        // RollupsFacet Claim event
        let rollups_claim_events =
            rollups_facet
                .claim_filter()
                .query()
                .await
                .context("Error querying for Rollups claim events")?;

        // step 1: `resolve_dispute_events`. For validator lost dispute, add to removal list and also remove address and #claims;
        //          for validator won, do nothing
        // step 2: if there are new_epoch_events, increase #claims for those in `claiming` but not in `validators_removed`.
        //          And clear `claiming`
        // step 3: for every finalized epoch, if a validator made a claim, and its address has not been removed, then #claims++
        // step 4: for epoch that hasn't been finalized (no more than 1 such epoch), store which honest validators have claimed

        // step 1:
        for ev in resolve_dispute_events.iter() {
            let losing_validator = ev.loser;
            state.validators_removed.push(losing_validator);
            // also need to clear it in num_claims
            for i in 0..MAX_NUM_VALIDATORS {
                if let Some(num_claims_struct) = &state.num_claims[i] {
                    let addr = num_claims_struct.validator_address;
                    if addr == losing_validator {
                        state.num_claims[i] = None;
                        break;
                    }
                }
            }
        }

        // step 2:
        let newly_finalized_epochs = U256::from(new_epoch_events.len());
        if newly_finalized_epochs > U256::zero() {
            state.num_finalized_epochs =
                state.num_finalized_epochs + newly_finalized_epochs;
            // increase #claims for those in `claiming`
            let claiming = state.claiming.clone();
            // because we want to pass `state` to `find_validator_and_increase()`
            // we don't want `state` to be borrowed by `iter()`, hence the `clone()`
            for v in claiming.iter() {
                if state.validators_removed.contains(v) {
                    continue;
                }
                find_validator_and_increase(*v, &mut state);
            }

            state.claiming.clear();
        }

        for ev in rollups_claim_events.iter() {
            let claim_epoch_num = ev.epoch_number;
            let v = ev.claimer;
            if state.validators_removed.contains(&v) {
                continue;
            }
            if claim_epoch_num < state.num_finalized_epochs {
                // step 3
                find_validator_and_increase(v, &mut state);
            } else {
                // step 4
                state.claiming.push(v);
            }
        }

        Ok(state)
    }
}

fn find_validator_and_increase(v: Address, state: &mut ValidatorManagerState) {
    // there can be `None` between `Some`
    let mut found = false;
    for i in 0..MAX_NUM_VALIDATORS {
        if let Some(num_claims_struct) = &state.num_claims[i] {
            let addr = num_claims_struct.validator_address;
            let num = num_claims_struct.num_claims_made;
            if addr == v {
                state.num_claims[i] = Some(NumClaims {
                    validator_address: addr,
                    num_claims_made: num + 1,
                });
                found = true;
                break;
            }
        }
    }
    if !found {
        // if not found, add to `num_claims`
        for i in 0..MAX_NUM_VALIDATORS {
            if let None = state.num_claims[i] {
                state.num_claims[i] = Some(NumClaims {
                    validator_address: v,
                    num_claims_made: U256::one(),
                });
                break;
            }
        }
    }
}
