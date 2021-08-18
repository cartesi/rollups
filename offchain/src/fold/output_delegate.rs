use crate::contracts::output_contract::*;

use super::types::OutputState;

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

use rpds::{Vector, VectorSync};

/// Output StateFold Delegate
pub struct OutputFoldDelegate {
    output_address: Address,
}

impl OutputFoldDelegate {
    pub fn new(output_address: Address) -> Self {
        Self { output_address }
    }
}

/// output_position = output_index * 2 ** 128 + input_index * 2 ** 64 + epoch
/// We always assume indices have at most 8 bytes, as does rust
fn convert_output_position_to_indices(output_position: U256) -> (usize, usize, usize) {
    let mut pos_bytes = [0u8; 32];
    output_position.to_big_endian(&mut pos_bytes);

    let mut output_index_bytes = [0u8; 8];
    output_index_bytes.copy_from_slice(&pos_bytes[8..16]);

    let mut input_index_bytes = [0u8; 8];
    input_index_bytes.copy_from_slice(&pos_bytes[16..24]);

    let mut epoch_bytes = [0u8; 8];
    epoch_bytes.copy_from_slice(&pos_bytes[24..32]);

    (
        usize::from_be_bytes(input_index_bytes),
        usize::from_be_bytes(output_index_bytes),
        usize::from_be_bytes(epoch_bytes),
    )
}

#[async_trait]
impl StateFoldDelegate for OutputFoldDelegate {
    type InitialState = ();
    type Accumulator = OutputState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        _initial_state: &(),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let contract = access
            .build_sync_contract(
                self.output_address,
                block.number,
                OutputImpl::new,
            )
            .await;

        // Retrieve `OutputExecuted` events
        let events = contract
            .output_executed_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for output executed events",
            })?;

        let mut outputs: VectorSync<VectorSync<VectorSync<bool>>> = Vector::new_sync();
        for ev in events {
            let (output_index, input_index, epoch) =
                convert_output_position_to_indices(ev.output_position);
            outputs[output_index][input_index][epoch] = true;
        }

        Ok(OutputState {
            outputs,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        // If not in bloom copy previous state
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &self.output_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &OutputExecutedFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                self.output_address,
                block.hash,
                OutputImpl::new,
            )
            .await;

        let events = contract
            .output_executed_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for output executed events",
            })?;

        let mut outputs = previous_state.outputs.clone();
        for ev in events {
            let (output_index, input_index, epoch_index) =
                convert_output_position_to_indices(ev.output_position);
            outputs[output_index][input_index][epoch_index] = true;
        }

        Ok(OutputState {
            outputs,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
