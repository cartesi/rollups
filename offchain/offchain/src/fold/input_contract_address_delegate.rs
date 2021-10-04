use crate::contracts::descartesv2_contract::*;

use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
};

use async_trait::async_trait;
use offchain_core::types::Block;

use ethers::types::Address;

use std::sync::Arc;

/// Input Contract Address Delegate, which implements `sync` and `fold`.
#[derive(Default)]
pub struct InputContractAddressFoldDelegate {}

#[derive(Clone, Debug)]
pub struct InputContractAddressState {
    pub input_contract_address: Address,
}

#[async_trait]
impl StateFoldDelegate for InputContractAddressFoldDelegate {
    type InitialState = Address;
    type Accumulator = InputContractAddressState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Address,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let descartesv2_contract_address = *initial_state;

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(
            descartesv2_contract_address,
            Arc::clone(&middleware),
        );

        let input_contract_address =
            contract.get_input_address().call().await.ok().unwrap();

        Ok(InputContractAddressState {
            input_contract_address,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        _block: &Block,
        _access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        Ok(InputContractAddressState {
            input_contract_address: previous_state.input_contract_address,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
