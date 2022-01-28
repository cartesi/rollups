// the following must be true:
// 1. a validator cannot claim multiple times in a single epoch
// 2. in any epoch, there is at least 1 claim

use crate::contracts::erc20_contract::*;
use crate::contracts::rollups_contract::*;
use crate::contracts::validator_manager_contract::*;

use super::types::ValidatorManagerState;

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
use ethers::types::{Address, H256, U256};

use num_enum::IntoPrimitive;

/// Validator Manager Delegate
#[derive(Default)]
pub struct ValidatorManagerFoldDelegate {}

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Result {
    NoConflict,
    Consensus,
    Conflict,
}

#[async_trait]
impl StateFoldDelegate for ValidatorManagerFoldDelegate {
    type InitialState = (Address, Address); // (validator manager address, rollups address)
    type Accumulator = ValidatorManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (validator_manager_address, rollups_address) = *initial_state;
        let validator_manager_contract = access
            .build_sync_contract(
                validator_manager_address,
                block.number,
                ValidatorManagerClaimsCountedImpl::new,
            )
            .await;
        let rollups_contract = access
            .build_sync_contract(
                rollups_address,
                block.number,
                RollupsImpl::new,
            )
            .await;

        // declare variable types
        let mut num_claims: [Option<(Address, U256)>; 8] = [None; 8];
        let mut validators_removed: Vec<Address> = Vec::new();
        let mut claiming: Vec<Address> = Vec::new(); // validators that have claimed in the current unfinalized epoch

        // retrive events

        // DisputeEnded event
        let mut dispute_ended_events = validator_manager_contract
            .dispute_ended_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for dispute ended events",
            })?;

        // NewEpoch event
        let mut new_epoch_events = validator_manager_contract
            .new_epoch_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for new epoch events",
            })?;

        // // ClaimReceived event (not needed, use rollups_claim_events instead)
        // let claim_received_events = validator_manager_contract
        //     .claim_received_filter()
        //     .query()
        //     .await
        //     .context(SyncContractError {
        //         err: "Error querying for claim received events",
        //     })?;

        // RollupsImpl Claim event
        let rollups_claim_events =
            rollups_contract.claim_filter().query().await.context(
                SyncContractError {
                    err: "Error querying for Rollups claim events",
                },
            )?;

        // step 1: `dispute_ended_events`. For validator lost dispute, add to removal list; for validator won, do nothing
        // step 2: for every finalized epoch, if a validator made a claim, and its address has not been removed, then #claims++.
        //          Those who made a false claim have been removed in step 1 already.
        // step 3: for epoch that hasn't been finalized (no more than 1 such epoch), store which validators have claimed
        //          No need to store the claim content. Because the dishonest will be removed before epoch finalized

        // step 1:
        for ev in dispute_ended_events.iter() {
            let losing_validator = ev.validators[1];
            validators_removed.push(losing_validator);
        }

        // step 2&3:
        let num_finalized_epochs = U256::from(new_epoch_events.len());
        for ev in rollups_claim_events.iter() {
            let claim_epoch_num = ev.epoch_number;
            if claim_epoch_num < num_finalized_epochs {
                // step 2
                let claimer = ev.claimer;
                if !validators_removed.contains(&claimer) {
                    for i in 0..8 {
                        if let Some((addr, num)) = num_claims[i] {
                            if addr == claimer {
                                num_claims[i] = Some((addr, num + 1));
                                break;
                            } else {
                                continue;
                            }
                        }
                        if let None = num_claims[i] {
                            // at this stage, there's no `None` between `Some`
                            num_claims[i] = Some((claimer, U256::one()));
                        }
                    }
                } else {
                    continue;
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
            validator_manager_address,
            rollups_address,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let validator_manager_address =
            previous_state.validator_manager_address;
        let rollups_address = previous_state.rollups_address;
        // If not in bloom copy previous state
        if !((fold_utils::contains_address(
            &block.logs_bloom,
            &validator_manager_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimReceivedFilter::signature(), // this event can be ignored
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &DisputeEndedFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &NewEpochFilter::signature(),
        ))) || (fold_utils::contains_address(
            &block.logs_bloom,
            &rollups_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimFilter::signature(),
        )))) {
            return Ok(previous_state.clone());
        }

        let validator_manager_contract = access
            .build_fold_contract(
                validator_manager_address,
                block.hash,
                ValidatorManagerClaimsCountedImpl::new,
            )
            .await;

        let rollups_contract = access
            .build_fold_contract(rollups_address, block.hash, RollupsImpl::new)
            .await;

        let mut state = previous_state.clone();

        // retrive events

        // DisputeEnded event
        let mut dispute_ended_events = validator_manager_contract
            .dispute_ended_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for dispute ended events",
            })?;

        // NewEpoch event
        let mut new_epoch_events = validator_manager_contract
            .new_epoch_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for new epoch events",
            })?;

        // RollupsImpl Claim event
        let rollups_claim_events =
            rollups_contract.claim_filter().query().await.context(
                FoldContractError {
                    err: "Error querying for Rollups claim events",
                },
            )?;

        // step 1: `dispute_ended_events`. For validator lost dispute, add to removal list and remove address and #claims;
        //          for validator won, do nothing
        // step 2: if there are new_epoch_events, increase #claims for those in `claiming` but not in `validators_removed`.
        //          And clear `claiming`
        // step 3: for every finalized epoch, if a validator made a claim, and its address has not been removed, then #claims++
        // step 4: for epoch that hasn't been finalized (no more than 1 such epoch), store which validators have claimed

        // step 1:
        for ev in dispute_ended_events.iter() {
            let losing_validator = ev.validators[1];
            state.validators_removed.push(losing_validator);
            // also need to clear it in num_claims
            for i in 0..8 {
                if let Some((addr, num)) = state.num_claims[i] {
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
                state = find_validator_and_increase(*v, state);
            }

            state.claiming.clear();
        }

        // step 3&4:
        for ev in rollups_claim_events.iter() {
            let claim_epoch_num = ev.epoch_number;
            if claim_epoch_num < state.num_finalized_epochs {
                // step 3
                let v = ev.claimer;
                if state.validators_removed.contains(&v) {
                    continue;
                }
                state = find_validator_and_increase(v, state);
            } else {
                // step 4
                state.claiming.push(ev.claimer);
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

fn find_validator_and_increase(
    v: Address,
    mut state: ValidatorManagerState,
) -> ValidatorManagerState {
    // there can be `None` between `Some`
    let mut found = false;
    for i in 0..8 {
        if let Some((addr, num)) = state.num_claims[i] {
            if addr == v {
                state.num_claims[i] = Some((addr, num + 1));
                found = true;
                break;
            }
        }
    }
    if !found {
        for i in 0..8 {
            if let None = state.num_claims[i] {
                state.num_claims[i] = Some((v, U256::one()));
                break;
            }
        }
    }
    state
}
