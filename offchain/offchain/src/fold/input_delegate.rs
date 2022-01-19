use offchain_core::ethers;

use crate::contracts::input_contract::*;

use super::types::{EpochInputState, Input};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};

use async_trait::async_trait;
use im::Vector;
use snafu::ResultExt;
use std::sync::Arc;

use ethers::contract::LogMeta;
use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};

/// Input StateFold Delegate
#[derive(Default)]
pub struct InputFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for InputFoldDelegate {
    type InitialState = (Address, U256);
    type Accumulator = EpochInputState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (dapp_contract_address, epoch_number) = initial_state.clone();

        let contract = access
            .build_sync_contract(
                dapp_contract_address,
                block.number,
                InputImpl::new,
            )
            .await;

        // Retrieve `InputAdded` events
        let events = contract
            .input_added_filter()
            .topic1(epoch_number)
            .query_with_meta()
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
            dapp_contract_address,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let dapp_contract_address = previous_state.dapp_contract_address;
        // If not in bloom copy previous state
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &InputAddedFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                InputImpl::new,
            )
            .await;

        let events = contract
            .input_added_filter()
            .topic1(previous_state.epoch_number)
            .query_with_meta()
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
            dapp_contract_address,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl From<(InputAddedFilter, LogMeta)> for Input {
    fn from(log: (InputAddedFilter, LogMeta)) -> Self {
        let ev = log.0;
        Self {
            sender: ev.sender,
            payload: Arc::new(ev.input),
            timestamp: ev.timestamp,

            block_number: log.1.block_number,
        }
    }
}
