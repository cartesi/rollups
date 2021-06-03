use super::contracts::input_contract::*;
use super::types::{EpochInputState, Input};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::Vector;
use snafu::ResultExt;
use std::sync::Arc;

use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};

/// Input StateFold Delegate
pub struct InputFoldDelegate {
    input_address: Address,
}

impl InputFoldDelegate {
    pub fn new(input_address: Address) -> Self {
        Self { input_address }
    }
}

#[async_trait]
impl StateFoldDelegate for InputFoldDelegate {
    type InitialState = U256;
    type Accumulator = EpochInputState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let epoch_number = initial_state.clone();

        let contract = access
            .build_sync_contract(
                self.input_address,
                block.number,
                InputImpl::new,
            )
            .await;

        let events = contract
            .input_added_filter()
            .topic1(epoch_number)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for input added events",
            })?;

        let mut inputs: Vector<Input> = Vector::new();
        for ev in events {
            inputs.push_back(ev.into());
        }

        Ok(EpochInputState {
            epoch_number,
            inputs,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &self.input_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &InputAddedFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(self.input_address, block.hash, InputImpl::new)
            .await;

        let events = contract
            .input_added_filter()
            .topic1(previous_state.epoch_number)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for input added events",
            })?;

        let mut inputs = previous_state.inputs.clone();
        for ev in events {
            inputs.push_back(ev.into());
        }

        Ok(EpochInputState {
            epoch_number: previous_state.epoch_number,
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

impl From<InputAddedFilter> for Input {
    fn from(ev: InputAddedFilter) -> Self {
        Self {
            sender: ev.sender,
            payload: Arc::new(ev.input),
            timestamp: ev.timestamp,
        }
    }
}
