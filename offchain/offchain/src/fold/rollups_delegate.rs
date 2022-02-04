use offchain_core::ethers;

use crate::contracts::rollups_contract::*;

use super::epoch_delegate::{ContractPhase, EpochFoldDelegate, EpochState};
use super::output_delegate::OutputFoldDelegate;
use super::sealed_epoch_delegate::SealedEpochState;
use super::types::{
    AccumulatingEpoch, ImmutableState, OutputState, PhaseState, RollupsState,
};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    DelegateAccess, StateFold,
};

use async_trait::async_trait;
use snafu::ResultExt;
use std::sync::Arc;

use crate::fold::fee_manager_delegate::FeeManagerFoldDelegate;
use crate::fold::types::FeeManagerState;
use crate::fold::types::ValidatorManagerState;
use crate::fold::validator_manager_delegate::ValidatorManagerFoldDelegate;
use ethers::providers::Middleware;
use ethers::types::{Address, U256};

/// Rollups StateActor Delegate, which implements `sync` and `fold`.
pub struct RollupsFoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    epoch_fold: Arc<StateFold<EpochFoldDelegate<DA>, DA>>,
    output_fold: Arc<StateFold<OutputFoldDelegate, DA>>,
    validator_manager_fold: Arc<StateFold<ValidatorManagerFoldDelegate, DA>>,
    fee_manager_fold: Arc<StateFold<FeeManagerFoldDelegate<DA>, DA>>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> RollupsFoldDelegate<DA> {
    pub fn new(
        epoch_fold: Arc<StateFold<EpochFoldDelegate<DA>, DA>>,
        output_fold: Arc<StateFold<OutputFoldDelegate, DA>>,
        validator_manager_fold: Arc<
            StateFold<ValidatorManagerFoldDelegate, DA>,
        >,
        fee_manager_fold: Arc<StateFold<FeeManagerFoldDelegate<DA>, DA>>,
    ) -> Self {
        Self {
            epoch_fold,
            output_fold,
            validator_manager_fold,
            fee_manager_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for RollupsFoldDelegate<DA>
{
    type InitialState = (Address, U256);
    type Accumulator = RollupsState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(Address, U256),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let (rollups_contract_address, epoch_number) = *initial_state;

        let contract =
            RollupsImpl::new(rollups_contract_address, Arc::clone(&middleware));

        // Retrieve constants from contract creation event
        let constants = {
            let (create_event, meta) = {
                let e = contract
                    .rollups_created_filter()
                    .query_with_meta()
                    .await
                    .context(SyncContractError {
                        err: "Error querying for rollups created",
                    })?;

                if e.is_empty() {
                    return SyncDelegateError {
                        err: "Rollups create event not found",
                    }
                    .fail();
                }

                assert_eq!(e.len(), 1);
                e[0].clone()
            };

            // retrieve timestamp of creation
            let timestamp = middleware
                .get_block(meta.block_hash)
                .await
                .context(SyncAccessError {})?
                .ok_or(snafu::NoneError)
                .context(SyncDelegateError {
                    err: "Block not found",
                })?
                .timestamp;

            ImmutableState::from(&(
                create_event,
                timestamp,
                rollups_contract_address,
            ))
        };

        // get raw state from EpochFoldDelegate
        let raw_contract_state = self
            .epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, epoch_number),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let output_state = self
            .output_fold
            .get_state_for_block(
                &constants.output_contract_address,
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Output state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let validator_manager_state = self
            .validator_manager_fold
            .get_state_for_block(
                &(
                    constants.validator_contract_address,
                    constants.rollups_contract_address,
                ),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Validator Manager state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let fee_manager_state = self
            .fee_manager_fold
            .get_state_for_block(
                &constants.fee_manager_contract_address,
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Fee Manager state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        Ok(convert_raw_to_logical(
            raw_contract_state,
            constants,
            block,
            &epoch_number,
            output_state,
            validator_manager_state,
            fee_manager_state,
        ))
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        _access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let constants = previous_state.constants.clone();
        let output_address = constants.output_contract_address;
        let validator_manager_address = constants.validator_contract_address;
        let fee_manager_address = constants.fee_manager_contract_address;

        // get raw state from EpochFoldDelegate
        let raw_contract_state = self
            .epoch_fold
            .get_state_for_block(
                &(
                    constants.rollups_contract_address,
                    previous_state.initial_epoch,
                ),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let output_state = self
            .output_fold
            .get_state_for_block(&output_address, Some(block.hash))
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Output state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let validator_manager_state = self
            .validator_manager_fold
            .get_state_for_block(
                &(
                    validator_manager_address,
                    constants.rollups_contract_address,
                ),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Validator manager fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let fee_manager_state = self
            .fee_manager_fold
            .get_state_for_block(&fee_manager_address, Some(block.hash))
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Fee manager fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        Ok(convert_raw_to_logical(
            raw_contract_state,
            constants,
            block,
            &previous_state.initial_epoch,
            output_state,
            validator_manager_state,
            fee_manager_state,
        ))
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

// Convert raw state to logical state. Raw state is the literal interpretation
// of what is being presented by the blockchain. Logical state is the semantic
// intepretation of that, which will be used for offchain decision making
fn convert_raw_to_logical(
    contract_state: EpochState,
    constants: ImmutableState,
    block: &Block,
    initial_epoch: &U256,
    output_state: OutputState,
    validator_manager_state: ValidatorManagerState,
    fee_manager_state: FeeManagerState,
) -> RollupsState {
    // If the raw state is InputAccumulation but it has expired, then the raw
    // state's `current_epoch` becomes the sealed epoch, and the logic state's
    // `current_epoch` is empty.
    // This variable contains `Some(epoch_number)` in this case, and `None`
    // otherwise.
    // This is possible because a new input after InputAccumulation has expired
    // would trigger a phase change to AwaitingConsensus.
    let mut current_epoch_no_inputs: Option<U256> = None;

    let phase_state = match contract_state.current_phase {
        ContractPhase::InputAccumulation {} => {
            // Last phase change timestamp is the timestamp of input
            // accumulation start if contract in InputAccumulation.
            // If there were no phase changes, it is the timestamp of
            // contract creation.
            let input_accumulation_start_timestamp =
                if let Some(ts) = contract_state.phase_change_timestamp {
                    ts
                } else {
                    constants.contract_creation_timestamp
                };

            // If input duration has passed, the logical state is epoch sealed
            // awaiting first claim. The raw state can still be InputAccumulation
            // if there were no new inputs after the phase expired.
            if block.timestamp
                > input_accumulation_start_timestamp + constants.input_duration
            {
                current_epoch_no_inputs =
                    Some(contract_state.current_epoch.epoch_number + 1);
                PhaseState::EpochSealedAwaitingFirstClaim {
                    sealed_epoch: contract_state.current_epoch.clone(),
                }
            } else {
                PhaseState::InputAccumulation {}
            }
        }

        ContractPhase::AwaitingConsensus {
            sealed_epoch,
            round_start,
        } => {
            // The raw phase change might have happened because a claim arrived
            // or because a new input arrived. This determines if the logical
            // phase is EpochAwaintFirstClaim or SealedEpochNoClaims
            match sealed_epoch {
                SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                    PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch }
                }

                SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                    let first_claim_timestamp =
                        claimed_epoch.claims.first_claim_timestamp();

                    // We can safely unwrap because we can be sure
                    // there was at least one phase change event.
                    // let phase_change_timestamp =
                    //     contract_state.phase_change_timestamp.unwrap();
                    let phase_change_timestamp = round_start;

                    // Last move's timestamp is the most recent timestamp between
                    // the first claim or the phase change. This happens because
                    // the 'challenge period' starts on first claim but resets
                    // after a dispute.
                    let time_of_last_move = std::cmp::max(
                        first_claim_timestamp,
                        phase_change_timestamp,
                    );

                    // Check if Consensus timed out or, using the first claim
                    // timestamp variable, decide if this is the first challenge
                    // period of this epoch or if it is posterior to a dispute
                    if block.timestamp
                        > time_of_last_move + constants.challenge_period
                    {
                        PhaseState::ConsensusTimeout { claimed_epoch }
                    } else if time_of_last_move == first_claim_timestamp {
                        PhaseState::AwaitingConsensusNoConflict {
                            claimed_epoch,
                        }
                    } else {
                        PhaseState::AwaitingConsensusAfterConflict {
                            claimed_epoch,
                            challenge_period_base_ts: phase_change_timestamp,
                        }
                    }
                }
            }
        }

        // This version doesn't have disputes. They're resolved automatically
        // onchain
        ContractPhase::AwaitingDispute { .. } => {
            unreachable!()
        }
    };

    // Figures out if the current accumulating epoch is empty (new) or if it
    // was previously created. The distinction comes from the two possible
    // transitions to AwaitingConsensus, either a new input or a claim
    let current_epoch = if let Some(epoch_number) = current_epoch_no_inputs {
        AccumulatingEpoch::new(
            constants.rollups_contract_address,
            constants.input_contract_address,
            epoch_number,
        )
    } else {
        contract_state.current_epoch
    };

    RollupsState {
        constants,
        initial_epoch: *initial_epoch,
        current_phase: phase_state,
        finalized_epochs: contract_state.finalized_epochs,
        current_epoch,
        output_state,
        validator_manager_state,
        fee_manager_state,
    }
}

// Fetches the Rollups constants from the contract creation event
impl From<&(RollupsCreatedFilter, U256, Address)> for ImmutableState {
    fn from(src: &(RollupsCreatedFilter, U256, Address)) -> Self {
        let (ev, ts, rollups_contract_address) = src;
        Self {
            input_duration: ev.input_duration,
            challenge_period: ev.challenge_period,
            contract_creation_timestamp: ts.clone(),
            input_contract_address: ev.input,
            output_contract_address: ev.output,
            validator_contract_address: ev.validator_manager,
            dispute_contract_address: ev.dispute_manager,
            fee_manager_contract_address: ev.fee_manager,
            rollups_contract_address: *rollups_contract_address,
        }
    }
}
