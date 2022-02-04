use offchain_core::ethers;

use crate::contracts::rollups_contract::*;

use super::types::{AccumulatingEpoch, EpochWithClaims, FinalizedEpochs};

use super::{
    accumulating_epoch_delegate::AccumulatingEpochFoldDelegate,
    finalized_epoch_delegate::FinalizedEpochFoldDelegate,
    sealed_epoch_delegate::{SealedEpochFoldDelegate, SealedEpochState},
};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};

use async_trait::async_trait;
use snafu::ResultExt;
use std::sync::Arc;

use ethers::prelude::EthEvent;
use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};

///
#[derive(Clone, Debug)]
pub enum ContractPhase {
    InputAccumulation {},

    AwaitingConsensus {
        sealed_epoch: SealedEpochState,
        round_start: U256,
    },

    AwaitingDispute {
        sealed_epoch: EpochWithClaims,
    },
}

#[derive(Clone, Debug)]
pub struct EpochState {
    pub initial_epoch: U256,

    pub current_phase: ContractPhase,
    pub finalized_epochs: FinalizedEpochs,
    pub current_epoch: AccumulatingEpoch,

    // Timestamp of last contract phase change
    pub phase_change_timestamp: Option<U256>,

    rollups_contract_address: Address,
}

type AccumulatingEpochStateFold<DA> =
    Arc<StateFold<AccumulatingEpochFoldDelegate<DA>, DA>>;
type SealedEpochStateFold<DA> = Arc<StateFold<SealedEpochFoldDelegate<DA>, DA>>;
type FinalizedEpochStateFold<DA> =
    Arc<StateFold<FinalizedEpochFoldDelegate<DA>, DA>>;

/// Epoch StateActor Delegate, which implements `sync` and `fold`.
/// It uses the subdelegates to extracts the raw state from blockchain
/// emitted events
pub struct EpochFoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    accumulating_epoch_fold: AccumulatingEpochStateFold<DA>,
    sealed_epoch_fold: SealedEpochStateFold<DA>,
    finalized_epoch_fold: FinalizedEpochStateFold<DA>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> EpochFoldDelegate<DA> {
    pub fn new(
        accumulating_epoch_fold: AccumulatingEpochStateFold<DA>,
        sealed_epoch_fold: SealedEpochStateFold<DA>,
        finalized_epoch_fold: FinalizedEpochStateFold<DA>,
    ) -> Self {
        Self {
            accumulating_epoch_fold,
            sealed_epoch_fold,
            finalized_epoch_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for EpochFoldDelegate<DA>
{
    type InitialState = (Address, U256);
    type Accumulator = EpochState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(Address, U256),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (rollups_contract_address, initial_epoch) = *(initial_state);

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract =
            RollupsImpl::new(rollups_contract_address, Arc::clone(&middleware));

        // retrieve list of finalized epochs from FinalizedEpochFoldDelegate
        let finalized_epochs = self
            .finalized_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, initial_epoch),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Finalized epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        // The index of next epoch is the number of finalized epochs
        let next_epoch = finalized_epochs.next_epoch();

        // Retrieve events emitted by the blockchain on phase changes
        let phase_change_events = contract
            .phase_change_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for rollups phase change",
            })?;

        let phase_change_timestamp = {
            match phase_change_events.last() {
                None => None,
                Some((_, meta)) => Some(
                    middleware
                        .get_block(meta.block_hash)
                        .await
                        .context(SyncAccessError {})?
                        .ok_or(snafu::NoneError)
                        .context(SyncDelegateError {
                            err: "Block not found",
                        })?
                        .timestamp,
                ),
            }
        };

        // Define the current_phase and current_epoch  based on the last
        // phase_change event
        let (current_phase, current_epoch) = match phase_change_events.last() {
            // InputAccumulation
            // either accumulating inputs or sealed epoch with no claims/new inputs
            Some((PhaseChangeFilter { new_phase: 0 }, _)) | None => {
                let current_epoch = self
                    .get_acc_sync(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                (ContractPhase::InputAccumulation {}, current_epoch)
            }

            // AwaitingConsensus
            // can be SealedEpochNoClaims or SealedEpochWithClaims
            Some((PhaseChangeFilter { new_phase: 1 }, _)) => {
                let sealed_epoch = self
                    .get_sealed_sync(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                let current_epoch = self
                    .get_acc_sync(
                        rollups_contract_address,
                        next_epoch + 1,
                        block.hash,
                    )
                    .await?;

                // Unwrap is safe because, a phase change event guarantees
                // a phase change timestamp
                let round_start = phase_change_timestamp.unwrap();

                (
                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start,
                    },
                    current_epoch,
                )
            }

            // AwaitingDispute
            Some((PhaseChangeFilter { new_phase: 2 }, _)) => {
                let sealed_epoch = self
                    .get_sealed_sync(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                let current_epoch = self
                    .get_acc_sync(
                        rollups_contract_address,
                        next_epoch + 1,
                        block.hash,
                    )
                    .await?;

                (
                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match sealed_epoch {
                            // If there are no claims then the contract can't
                            // be in AwaitingDispute phase
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return SyncDelegateError {
                                    err: format!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                ),
                                }
                                .fail()
                            }
                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => claimed_epoch,
                        },
                    },
                    current_epoch,
                )
            }

            // Err
            Some((PhaseChangeFilter { new_phase }, _)) => {
                return SyncDelegateError {
                    err: format!(
                        "Could not convert new_phase `{}` to PhaseState",
                        new_phase
                    ),
                }
                .fail()
            }
        };

        Ok(EpochState {
            current_phase,
            phase_change_timestamp,
            initial_epoch,
            finalized_epochs,
            current_epoch,
            rollups_contract_address,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let rollups_contract_address = previous_state.rollups_contract_address;
        // Check if there was (possibly) some log emited on this block.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &rollups_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &PhaseChangeFilter::signature(),
        )) {
            // Current phase has not changed, but we need to update the
            // sub-states.
            let current_epoch = self
                .get_acc_fold(
                    rollups_contract_address,
                    previous_state.current_epoch.epoch_number,
                    block.hash,
                )
                .await?;

            let current_phase = match &previous_state.current_phase {
                ContractPhase::InputAccumulation {} => {
                    ContractPhase::InputAccumulation {}
                }

                ContractPhase::AwaitingConsensus {
                    sealed_epoch,
                    round_start,
                } => {
                    let sealed_epoch = self
                        .get_sealed_fold(
                            rollups_contract_address,
                            sealed_epoch.epoch_number(),
                            block.hash,
                        )
                        .await?;

                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start: *round_start,
                    }
                }

                ContractPhase::AwaitingDispute { sealed_epoch } => {
                    let sealed_epoch = self
                        .get_sealed_fold(
                            rollups_contract_address,
                            sealed_epoch.epoch_number,
                            block.hash,
                        )
                        .await?;

                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match sealed_epoch {
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return FoldDelegateError {
                                    err: format!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                ),
                                }
                                .fail()
                            }
                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => claimed_epoch,
                        },
                    }
                }
            };

            return Ok(EpochState {
                current_phase,
                current_epoch,
                phase_change_timestamp: previous_state.phase_change_timestamp,
                initial_epoch: previous_state.initial_epoch,
                finalized_epochs: previous_state.finalized_epochs.clone(),
                rollups_contract_address,
            });
        }

        let contract = access
            .build_fold_contract(
                rollups_contract_address,
                block.hash,
                RollupsImpl::new,
            )
            .await;

        let finalized_epochs = self
            .finalized_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, previous_state.initial_epoch),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Finalized epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let next_epoch = finalized_epochs.next_epoch();

        let phase_change_events =
            contract.phase_change_filter().query().await.context(
                FoldContractError {
                    err: "Error querying for rollups phase change",
                },
            )?;

        let (current_phase, current_epoch) = match phase_change_events.last() {
            // InputAccumulation
            Some(PhaseChangeFilter { new_phase: 0 }) | None => {
                let current_epoch = self
                    .get_acc_fold(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                (ContractPhase::InputAccumulation {}, current_epoch)
            }

            // AwaitingConsensus
            Some(PhaseChangeFilter { new_phase: 1 }) => {
                // If the phase is AwaitingConsensus then there are two epochs
                // not yet finalized. One sealead, which can't receive new
                // inputs and one active, accumulating new inputs
                let sealed_epoch = self
                    .get_sealed_fold(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                let current_epoch = self
                    .get_acc_fold(
                        rollups_contract_address,
                        next_epoch + 1,
                        block.hash,
                    )
                    .await?;

                // Timestamp of when we entered this phase.
                let round_start = block.timestamp;

                (
                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start,
                    },
                    current_epoch,
                )
            }

            // AwaitingDispute
            Some(PhaseChangeFilter { new_phase: 2 }) => {
                // If the phase is AwaitingDispute then there are two epochs
                // not yet finalized. One sealead, which can't receive new
                // inputs and one active, accumulating new inputs
                let sealed_epoch = self
                    .get_sealed_fold(
                        rollups_contract_address,
                        next_epoch,
                        block.hash,
                    )
                    .await?;
                let current_epoch = self
                    .get_acc_fold(
                        rollups_contract_address,
                        next_epoch + 1,
                        block.hash,
                    )
                    .await?;

                (
                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match sealed_epoch {
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return FoldDelegateError {
                                    err: format!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                ),
                                }
                                .fail()
                            }
                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => claimed_epoch,
                        },
                    },
                    current_epoch,
                )
            }

            // Err
            Some(PhaseChangeFilter { new_phase }) => {
                return FoldDelegateError {
                    err: format!(
                        "Could not convert new_phase `{}` to PhaseState",
                        new_phase
                    ),
                }
                .fail()
            }
        };

        let phase_change_timestamp = if phase_change_events.is_empty() {
            previous_state.phase_change_timestamp
        } else {
            Some(block.timestamp)
        };

        Ok(EpochState {
            current_phase,
            current_epoch,
            phase_change_timestamp,
            initial_epoch: previous_state.initial_epoch,
            finalized_epochs,
            rollups_contract_address,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> EpochFoldDelegate<DA> {
    // Get result of AccumulatingEpoch sync call
    async fn get_acc_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        rollups_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<AccumulatingEpoch, A> {
        Ok(self
            .accumulating_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, epoch),
                Some(block_hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!(
                        "Accumulating epoch state fold error: {:?}",
                        e
                    ),
                }
                .build()
            })?
            .state)
    }

    // get result of AccumulatingEpoch fold call
    async fn get_acc_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        rollups_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<AccumulatingEpoch, A> {
        Ok(self
            .accumulating_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, epoch),
                Some(block_hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!(
                        "Accumulating epoch state fold error: {:?}",
                        e
                    ),
                }
                .build()
            })?
            .state)
    }

    // Get result of SealedEpoch sync call
    async fn get_sealed_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        rollups_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<SealedEpochState, A> {
        Ok(self
            .sealed_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, epoch),
                Some(block_hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Sealed epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }

    // Get result of SealedEpoch fold call
    async fn get_sealed_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        rollups_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<SealedEpochState, A> {
        Ok(self
            .sealed_epoch_fold
            .get_state_for_block(
                &(rollups_contract_address, epoch),
                Some(block_hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Sealed epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }
}
