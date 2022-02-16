use crate::contracts::rollups_facet::*;
use crate::contracts::validator_manager_facet::*;

use super::types::{NumClaims, ValidatorManagerState};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};

use async_trait::async_trait;
use snafu::ResultExt;

use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};

/// Validator Manager Delegate
#[derive(Default)]
pub struct ValidatorManagerFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for ValidatorManagerFoldDelegate {
    type InitialState = Address;
    type Accumulator = ValidatorManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let dapp_contract_address = *initial_state;
        let validator_manager_facet = access
            .build_sync_contract(
                dapp_contract_address,
                block.number,
                ValidatorManagerFacet::new,
            )
            .await;
        let rollups_facet = access
            .build_sync_contract(
                dapp_contract_address,
                block.number,
                RollupsFacet::new,
            )
            .await;

        // declare variables
        let mut num_claims: [Option<NumClaims>; 8] = [None; 8];
        let mut validators_removed: Vec<Address> = Vec::new();
        let mut claiming: Vec<Address> = Vec::new(); // validators that have claimed in the current unfinalized epoch

        // retrive events

        // RollupsFacet ResolveDispute event
        let resolve_dispute_events = rollups_facet
            .resolve_dispute_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for resolve dispute events",
            })?;

        // NewEpoch event
        let new_epoch_events = validator_manager_facet
            .new_epoch_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for new epoch events",
            })?;

        // RollupsFacet Claim event
        let rollups_claim_events =
            rollups_facet.claim_filter().query().await.context(
                SyncContractError {
                    err: "Error querying for Rollups claim events",
                },
            )?;

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
                for i in 0..8 {
                    // find claimer in `num_claims`
                    if let Some(num_claims_struct) = &num_claims[i] {
                        let addr = num_claims_struct.validator_address;
                        let num = num_claims_struct.num_claims_mades;
                        if addr == claimer {
                            num_claims[i] = Some(NumClaims {
                                validator_address: addr,
                                num_claims_mades: num + 1,
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
                            num_claims_mades: U256::one(),
                        });
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

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
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

        let validator_manager_facet = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                ValidatorManagerFacet::new,
            )
            .await;

        let rollups_facet = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                RollupsFacet::new,
            )
            .await;

        let mut state = previous_state.clone();

        // retrive events

        // RollupsFacet ResolveDispute event
        let resolve_dispute_events = rollups_facet
            .resolve_dispute_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for resolve dispute events",
            })?;

        // NewEpoch event
        let new_epoch_events = validator_manager_facet
            .new_epoch_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for new epoch events",
            })?;

        // RollupsFacet Claim event
        let rollups_claim_events =
            rollups_facet.claim_filter().query().await.context(
                FoldContractError {
                    err: "Error querying for Rollups claim events",
                },
            )?;

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
            for i in 0..8 {
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

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

fn find_validator_and_increase(v: Address, state: &mut ValidatorManagerState) {
    // there can be `None` between `Some`
    let mut found = false;
    for i in 0..8 {
        if let Some(num_claims_struct) = &state.num_claims[i] {
            let addr = num_claims_struct.validator_address;
            let num = num_claims_struct.num_claims_mades;
            if addr == v {
                state.num_claims[i] = Some(NumClaims {
                    validator_address: addr,
                    num_claims_mades: num + 1,
                });
                found = true;
                break;
            }
        }
    }
    if !found {
        // if not found, add to `num_claims`
        for i in 0..8 {
            if let None = state.num_claims[i] {
                state.num_claims[i] = Some(NumClaims {
                    validator_address: v,
                    num_claims_mades: U256::one(),
                });
                break;
            }
        }
    }
}
