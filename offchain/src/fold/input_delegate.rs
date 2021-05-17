use super::contracts::input_contract::*;

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

use ethers::types::{Address, U256};

#[derive(Clone, Debug)]
pub struct Input {
    pub sender: Address,       // TODO: Get from calldata.
    pub timestamp: U256,       // TODO: Get from calldata.
    pub payload: Arc<Vec<u8>>, // TODO: Get from calldata.
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub input_contract_address: Address,
    pub epoch: U256,
    pub inputs: Vector<Input>,
}

/// Input StateFold Delegate
pub struct InputFoldDelegate {}

impl InputFoldDelegate {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StateFoldDelegate for InputFoldDelegate {
    type InitialState = (Address, U256);
    type Accumulator = InputState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(Address, U256),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (input_contract_address, epoch) = initial_state.clone();

        let contract = access
            .build_sync_contract(
                input_contract_address,
                block.number,
                InputImpl::new,
            )
            .await;

        let events = contract
            .input_added_filter()
            .topic1(epoch)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for input added events",
            })?;

        let mut inputs: Vector<Input> = Vector::new();
        for ev in events {
            inputs.push_back(ev.into());
        }

        Ok(InputState {
            input_contract_address,
            epoch,
            inputs,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        if fold_utils::contains_address(
            &block.logs_bloom,
            &previous_state.input_contract_address,
        ) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                previous_state.input_contract_address,
                block.hash,
                InputImpl::new,
            )
            .await;

        let events = contract
            .input_added_filter()
            .topic1(previous_state.epoch)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for input added events",
            })?;

        let mut inputs = previous_state.inputs.clone();
        for ev in events {
            inputs.push_back(ev.into());
        }

        Ok(InputState {
            input_contract_address: previous_state.input_contract_address,
            epoch: previous_state.epoch,
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
