use offchain_core::ethers;

use crate::contracts::rollups_contract::*;

use super::input_delegate::InputFoldDelegate;
use super::types::{EpochInputState, FinalizedEpoch, FinalizedEpochs};

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
use ethers::types::{Address, H256, U256};

/// Finalized epoch StateFold Delegate
pub struct FinalizedEpochFoldDelegate<DA: DelegateAccess> {
    input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
}

impl<DA: DelegateAccess> FinalizedEpochFoldDelegate<DA> {
    pub fn new(
        input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
    ) -> Self {
        Self {
            input_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for FinalizedEpochFoldDelegate<DA>
{
    type InitialState = (Address, U256);
    type Accumulator = FinalizedEpochs;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(Address, U256),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (dapp_contract_address, initial_epoch) = *initial_state;

        let contract = access
            .build_sync_contract(
                dapp_contract_address,
                block.number,
                RollupsImpl::new,
            )
            .await;

        // Retrieve FinalizeEpoch events
        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for rollups finalized epochs",
            })?;

        let mut finalized_epochs = FinalizedEpochs::new(
            initial_epoch,
            dapp_contract_address,
        );

        // If number of epoch finalized events is smaller than the specified
        // `inital_epoch` then no update is needed
        if epoch_finalized_events.len() < initial_epoch.as_usize() {
            return Ok(finalized_epochs);
        }

        let slice = &epoch_finalized_events[initial_epoch.as_usize()..];
        // For every event in `epoch_finalized_events`, considering the
        // `initial_epoch` slice, add a `FinalizedEpoch` to the list
        for (ev, meta) in slice {
            let inputs = self
                .get_inputs_sync(
                    dapp_contract_address,
                    ev.epoch_number,
                    block.hash,
                )
                .await?;

            let epoch = FinalizedEpoch {
                epoch_number: ev.epoch_number,
                hash: ev.epoch_hash.into(),
                inputs,
                finalized_block_hash: meta.block_hash,
                finalized_block_number: meta.block_number,
            };

            let inserted = finalized_epochs.insert_epoch(epoch);
            assert!(inserted);
        }

        Ok(finalized_epochs)
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let dapp_contract_address =
            previous_state.dapp_contract_address;

        // Check if there was (possibly) some log emited on this block.
        // As finalized epochs' inputs will not change, we can return early
        // without querying the input StateFold.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &previous_state.next_epoch(),
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &FinalizeEpochFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                RollupsImpl::new,
            )
            .await;

        // Retrieve finalized epoch events
        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context(FoldContractError {
                err: "Error querying for rollups finalized epochs",
            })?;

        // Clone previous finalized epochs to the current list
        let mut finalized_epochs = previous_state.clone();

        // For every event create a new `FinalizedEpoch` and add it
        // to the list
        for (ev, meta) in epoch_finalized_events {
            if ev.epoch_number < finalized_epochs.next_epoch() {
                continue;
            }

            let inputs = self
                .get_inputs_fold(
                    dapp_contract_address,
                    ev.epoch_number,
                    block.hash,
                )
                .await?;

            let epoch = FinalizedEpoch {
                epoch_number: ev.epoch_number,
                hash: ev.epoch_hash.into(),
                inputs,
                finalized_block_hash: meta.block_hash,
                finalized_block_number: meta.block_number,
            };

            let inserted = finalized_epochs.insert_epoch(epoch);
            assert!(inserted);
        }

        Ok(finalized_epochs)
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static>
    FinalizedEpochFoldDelegate<DA>
{
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        dapp_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<EpochInputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(
                &(dapp_contract_address, epoch),
                Some(block_hash),
            )
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
        dapp_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<EpochInputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(
                &(dapp_contract_address, epoch),
                Some(block_hash),
            )
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
