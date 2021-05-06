use super::contracts::descartesv2_contract::*;

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::{HashMap, HashSet, OrdMap};
use snafu::ResultExt;
use std::convert::{TryFrom, TryInto};

use ethers::types::{Address, H256, U256};

#[derive(Clone, Debug)]
pub struct Input {
    pub sender: Address,
    pub hash: H256,
    pub timestamp: U256,
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub input_address: Address,
    pub epoch: U256,
    pub inputs: OrdMap<U256, Input>,
}

/// Partition StateActor Delegate, which implements `sync` and `fold`.
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
        todo!()
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
