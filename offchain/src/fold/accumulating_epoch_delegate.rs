use super::input_delegate::InputFoldDelegate;
use super::types::{AccumulatingEpoch, InputState};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    DelegateAccess, StateFold,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use std::sync::Arc;

use ethers::types::{Address, H256, U256};

/// Accumulating epoch StateFold Delegate
pub struct AccumulatingEpochFoldDelegate<DA: DelegateAccess> {
    descartesv2_address: Address,
    input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
}

impl<DA: DelegateAccess> AccumulatingEpochFoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
    ) -> Self {
        Self {
            descartesv2_address,
            input_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for AccumulatingEpochFoldDelegate<DA>
{
    type InitialState = U256;
    type Accumulator = AccumulatingEpoch;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let epoch_number = initial_state.clone();

        // Inputs of epoch
        let inputs = self.get_inputs_sync(epoch_number, block.hash).await?;

        Ok(AccumulatingEpoch {
            inputs,
            epoch_number,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let epoch_number = previous_state.epoch_number.clone();

        // Inputs of epoch
        let inputs = self.get_inputs_fold(epoch_number, block.hash).await?;

        Ok(AccumulatingEpoch {
            epoch_number,
            inputs,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static>
    AccumulatingEpochFoldDelegate<DA>
{
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&epoch, block_hash)
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
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&epoch, block_hash)
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
