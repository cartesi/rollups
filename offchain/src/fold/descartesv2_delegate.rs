use super::contracts::descartesv2_contract::*;
use super::epoch_delegate::{ContractPhase, EpochFoldDelegate};
use super::sealed_epoch_delegate::SealedEpochState;
use super::types::{
    AccumulatingEpoch, DescartesV2State, EpochWithClaims, FinalizedEpochs,
    ImmutableState, PhaseState,
};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::{HashMap, HashSet, Vector};
use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

use ethers::contract::LogMeta;
use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};

/// DescartesV2 StateActor Delegate, which implements `sync` and `fold`.
pub struct DescartesV2FoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    descartesv2_address: Address,
    epoch_fold: StateFold<EpochFoldDelegate<DA>, DA>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> DescartesV2FoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        epoch_fold: StateFold<EpochFoldDelegate<DA>, DA>,
    ) -> Self {
        Self {
            descartesv2_address,
            epoch_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for DescartesV2FoldDelegate<DA>
{
    type InitialState = U256; // Initial epoch
    type Accumulator = DescartesV2State;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(
            self.descartesv2_address,
            Arc::clone(&middleware),
        );

        let constants = {
            let (create_event, meta) = {
                let e = contract
                    .descartes_v2_created_filter()
                    .query_with_meta()
                    .await
                    .context(SyncContractError {
                        err: "Error querying for descartes created",
                    })?;

                if e.is_empty() {
                    return SyncDelegateError {
                        err: "Descartes create event not found",
                    }
                    .fail();
                }

                assert_eq!(e.len(), 1);
                e[0]
            };

            let timestamp = middleware
                .get_block(meta.block_hash)
                .await
                .context(SyncAccessError {})?
                .ok_or(snafu::NoneError)
                .context(SyncDelegateError {
                    err: "Block not found",
                })?
                .timestamp;

            ImmutableState::from(&(create_event, timestamp))
        };

        let contract_state = self
            .epoch_fold
            .get_state_for_block(initial_state, block.hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let mut is_current_epoch_empty = false;
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

                if block.timestamp
                    > input_accumulation_start_timestamp
                        + constants.input_duration
                {
                    is_current_epoch_empty = true;
                    PhaseState::EpochSealedAwaitingFirstClaim {
                        sealed_epoch: contract_state.current_epoch,
                    }
                } else {
                    PhaseState::InputAccumulation {}
                }
            }

            ContractPhase::AwaitingConsensus {
                sealed_epoch,
                round_start,
            } => {
                match sealed_epoch {
                    SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                        PhaseState::EpochSealedAwaitingFirstClaim {
                            sealed_epoch,
                        }
                    }

                    SealedEpochState::SealedEpochWithClaims {
                        claimed_epoch,
                    } => {
                        let first_claim_timestamp =
                            claimed_epoch.claims.first_claim_timestamp();

                        // We can safely unwrap because we can be sure
                        // there was at least one phase change event.
                        let phase_change_timestamp =
                            contract_state.phase_change_timestamp.unwrap();

                        let time_of_last_move = std::cmp::max(
                            first_claim_timestamp,
                            phase_change_timestamp,
                        );

                        // Check
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
                                challenge_period_base_ts:
                                    phase_change_timestamp,
                            }
                        }
                    }
                }
            }

            ContractPhase::AwaitingDispute { sealed_epoch } => {
                unreachable!()
            }
        };

        Ok(DescartesV2State {
            constants,
            initial_epoch: *initial_state,
            current_phase: phase_state,
            finalized_epochs: contract_state.finalized_epochs,
            current_epoch: contract_state.current_epoch,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let contract = access
            .build_fold_contract(
                self.descartesv2_address,
                block.hash,
                DescartesV2Impl::new,
            )
            .await;

        let constants = previous_state.constants.clone();

        let first_non_finalized_epoch_inputs = {
            // Index of first non finalized epoch
            let number = finalized_epochs.len().into();

            // Inputs of first non finalized epoch
            let inputs = self
                .get_inputs_fold(
                    constants.input_contract_address,
                    number,
                    block.hash,
                )
                .await?;

            AccumulatingEpoch { inputs, number }
        };

        let current_phase = {
            if fold_utils::contains_address(
                &block.logs_bloom,
                &self.descartesv2_address,
            ) {
                let phase_change_events =
                    contract.phase_change_filter().query().await.context(
                        FoldContractError {
                            err: "Error querying for descartes phase change",
                        },
                    )?;

                if let Some(p) = phase_change_events.last() {
                    match p.new_phase {
                        0 => PhaseState::InputAccumulation {
                            current_epoch: first_non_finalized_epoch_inputs,
                        },
                        1 => PhaseState::AwaitingConsensus,
                        2 => PhaseState::AwaitingDispute,
                        _ => {
                            return Err(format!(
                            "Could not convert new_phase `{}` to PhaseState",
                            p.new_phase
                        ))
                        }
                    }
                    // PhaseState::try_from(p)
                    //     .map_err(|err| FoldDelegateError { err }.build())?
                } else {
                    previous_state.current_phase.clone()
                }
            } else {
                previous_state.current_phase.clone()
            }
        };

        let finalized_epochs = {
            let epoch_finalized_events =
                contract.finalize_epoch_filter().query().await.context(
                    FoldContractError {
                        err: "Error querying for descartes finalized epochs",
                    },
                )?;

            let mut finalized_epochs: OrdMap<U256, _> =
                previous_state.finalized_epochs.clone();

            for (epoch, ev) in epoch_finalized_events.iter().enumerate() {
                let inputs = self
                    .get_inputs_fold(
                        constants.input_address,
                        epoch.into(),
                        block.hash,
                    )
                    .await?;

                finalized_epochs = finalized_epochs.update(
                    epoch.into(),
                    FinalizedEpoch {
                        hash: ev.epoch_hash.into(),
                        inputs,
                    },
                )
            }
            finalized_epochs
        };

        let current_epoch = {
            let number = finalized_epochs.len().into();

            let inputs = self
                .get_inputs_fold(constants.input_address, number, block.hash)
                .await?;

            let claim_events = contract
                .claim_filter()
                .topic1(U256::from(number))
                .query()
                .await
                .context(FoldContractError {
                    err: "Error querying for descartes phase change",
                })?;

            let mut claimers = if previous_state.current_epoch.number == number
            {
                previous_state.current_epoch.claimers.clone()
            } else {
                HashMap::new()
            };

            for claim in claim_events {
                let set = match claimers.get(&H256::from(claim.epoch_hash)) {
                    None => HashSet::new().update(claim.claimer),
                    Some(set) => set.update(claim.claimer),
                };

                claimers = claimers.update(claim.epoch_hash.into(), set)
            }

            CurrentEpoch {
                number,
                claimers,
                inputs,
            }
        };

        Ok(DescartesV2State {
            constants,
            current_phase,
            initial_epoch: previous_state.initial_epoch,
            finalized_epochs,
            current_epoch,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> DescartesV2FoldDelegate<DA> {
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        input_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&(input_address, epoch), block_hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Input state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }

    async fn get_inputs_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        input_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&(input_address, epoch), block_hash)
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Input state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }
}

// impl std::convert::TryFrom<&PhaseChangeFilter> for PhaseState {
//     type Error = String;
//     fn try_from(
//         p: &PhaseChangeFilter,
//     ) -> std::result::Result<Self, Self::Error> {
//         match p.new_phase {
//             0 => Ok(PhaseState::InputAccumulation),
//             1 => Ok(PhaseState::AwaitingConsensus),
//             2 => Ok(PhaseState::AwaitingDispute),
//             _ => Err(format!(
//                 "Could not convert new_phase `{}` to PhaseState",
//                 p.new_phase
//             )),
//         }
//     }
// }

impl From<&(DescartesV2CreatedFilter, U256)> for ImmutableState {
    fn from(src: &(DescartesV2CreatedFilter, U256)) -> Self {
        let (ev, ts) = src;
        Self {
            input_duration: ev.input_duration,
            challenge_period: ev.challenge_period,
            contract_creation_timestamp: ts.clone(),
            input_contract_address: ev.input,
            output_contract_address: ev.output,
            validator_contract_address: ev.validator_manager,
            dispute_contract_address: ev.dispute_manager,
        }
    }
}
