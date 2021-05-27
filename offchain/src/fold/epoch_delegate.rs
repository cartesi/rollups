use super::contracts::descartesv2_contract::*;

use super::input_delegate::InputFoldDelegate;
use super::types::{
    AccumulatingEpoch, Claims, FinalizedEpochs, InputState, SealedEpoch,
};
use super::{
    accumulating_epoch_delegate::AccumulatingEpochFoldDelegate,
    finalized_epoch_delegate::FinalizedEpochFoldDelegate,
    sealed_epoch_delegate::SealedEpochFoldDelegate,
};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::Vector;
use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};

///
#[derive(Clone, Debug)]
pub enum ContractPhase {
    InputAccumulation {
        current_epoch: AccumulatingEpoch,
    },

    AwaitingConsensus {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
        round_start: U256,
    },

    AwaitingDispute {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },
}

#[derive(Clone, Debug)]
pub struct EpochState {
    pub current_phase: ContractPhase,
    pub initial_epoch: U256,
    pub finalized_epochs: FinalizedEpochs,
}

type AccumulatingEpochStateFold<DA: DelegateAccess> =
    Arc<StateFold<AccumulatingEpochFoldDelegate<DA>, DA>>;

type SealedEpochStateFold<DA: DelegateAccess> =
    Arc<StateFold<SealedEpochFoldDelegate<DA>, DA>>;

type FinalizedEpochStateFold<DA: DelegateAccess> =
    Arc<StateFold<FinalizedEpochFoldDelegate<DA>, DA>>;

/// Epoch StateActor Delegate, which implements `sync` and `fold`.
pub struct EpochFoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    descartesv2_address: Address,
    accumulating_epoch_fold: AccumulatingEpochStateFold<DA>,
    sealed_epoch_fold: SealedEpochStateFold<DA>,
    finalized_epoch_fold: FinalizedEpochStateFold<DA>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> EpochFoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        accumulating_epoch_fold: AccumulatingEpochStateFold<DA>,
        sealed_epoch_fold: SealedEpochStateFold<DA>,
        finalized_epoch_fold: FinalizedEpochStateFold<DA>,
    ) -> Self {
        Self {
            descartesv2_address,
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
    type InitialState = U256;
    type Accumulator = EpochState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let initial_epoch = *initial_state;

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(
            self.descartesv2_address,
            Arc::clone(&middleware),
        );

        let finalized_epochs = self
            .finalized_epoch_fold
            .get_state_for_block(&initial_epoch, block.hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Finalized epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let next_epoch = finalized_epochs.next_epoch();

        let phase_change_events = contract
            .phase_change_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes phase change",
            })?;

        let current_phase = match phase_change_events.last() {
            // InputAccumulation
            Some((PhaseChangeFilter { new_phase: 0 }, _)) | None => {
                let current_epoch =
                    self.get_acc_sync(&next_epoch, block.hash).await?;
                ContractPhase::InputAccumulation { current_epoch }
            }

            // AwaitingConsensus
            Some((PhaseChangeFilter { new_phase: 1 }, m)) => {
                let sealed_epoch =
                    self.get_sealed_sync(&next_epoch, block.hash).await?;
                let current_epoch =
                    self.get_acc_sync(&(next_epoch + 1), block.hash).await?;

                // Timestamp of when we entered this phase.
                let round_start = middleware
                    .get_block(m.block_hash)
                    .await
                    .context(SyncAccessError {})?
                    .ok_or(snafu::NoneError)
                    .context(SyncDelegateError {
                        err: "Block not found",
                    })?
                    .timestamp;

                ContractPhase::AwaitingConsensus {
                    sealed_epoch,
                    current_epoch,
                    round_start,
                }
            }

            // AwaitingDispute
            Some((PhaseChangeFilter { new_phase: 2 }, m)) => {
                let sealed_epoch =
                    self.get_sealed_sync(&next_epoch, block.hash).await?;
                let current_epoch =
                    self.get_acc_sync(&(next_epoch + 1), block.hash).await?;

                ContractPhase::AwaitingDispute {
                    sealed_epoch,
                    current_epoch,
                }
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
            initial_epoch,
            finalized_epochs,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        todo!()
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> EpochFoldDelegate<DA> {
    async fn get_acc_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: &U256,
        block_hash: H256,
    ) -> SyncResult<AccumulatingEpoch, A> {
        Ok(self
            .accumulating_epoch_fold
            .get_state_for_block(epoch, block_hash)
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

    async fn get_acc_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        epoch: &U256,
        block_hash: H256,
    ) -> FoldResult<AccumulatingEpoch, A> {
        Ok(self
            .accumulating_epoch_fold
            .get_state_for_block(epoch, block_hash)
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

    async fn get_sealed_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: &U256,
        block_hash: H256,
    ) -> SyncResult<SealedEpoch, A> {
        Ok(self
            .sealed_epoch_fold
            .get_state_for_block(epoch, block_hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Sealed epoch state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }

    async fn get_sealed_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        epoch: &U256,
        block_hash: H256,
    ) -> FoldResult<SealedEpoch, A> {
        Ok(self
            .sealed_epoch_fold
            .get_state_for_block(epoch, block_hash)
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
