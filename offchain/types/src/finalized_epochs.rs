use crate::{
    epoch_initial_state::EpochInitialState, input::EpochInputState,
    rollups_initial_state::RollupsInitialState, FoldableError,
};
use anyhow::Context;
use async_trait::async_trait;
use contracts::rollups_facet::*;
use ethers::{
    prelude::EthEvent,
    providers::Middleware,
    types::{H256, U256, U64},
};
use im::Vector;
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

/// Epoch finalized on the blockchain, vouchers are executable and notices
/// are verifiable/provable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalizedEpoch {
    pub epoch_number: U256,
    pub hash: H256,
    pub inputs: Arc<EpochInputState>,

    /// Hash of block in which epoch was finalized
    pub finalized_block_hash: H256,

    /// Number of block in which epoch was finalized
    pub finalized_block_number: U64,
}

/// Set of finalized epochs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalizedEpochs {
    /// Set of `FinalizedEpoch`
    pub finalized_epochs: Vector<Arc<FinalizedEpoch>>,
    pub rollups_initial_state: Arc<RollupsInitialState>,
}

impl FinalizedEpochs {
    pub fn new(rollups_initial_state: Arc<RollupsInitialState>) -> Self {
        Self {
            finalized_epochs: Vector::new(),
            rollups_initial_state,
        }
    }

    pub fn get_epoch(&self, index: usize) -> Option<Arc<FinalizedEpoch>> {
        let initial_epoch = self.rollups_initial_state.initial_epoch.as_usize();

        if index >= initial_epoch && index < self.next_epoch().as_usize() {
            let actual_index = index - initial_epoch;
            Some(self.finalized_epochs[actual_index].clone())
        } else {
            None
        }
    }

    pub fn next_epoch(&self) -> U256 {
        self.rollups_initial_state.initial_epoch + self.finalized_epochs.len()
    }

    fn epoch_number_consistent(&self, epoch_number: &U256) -> bool {
        *epoch_number == self.next_epoch()
    }

    /// If `finalized_epoch.epoch_number` is not consistent, this method fails
    /// to insert epoch and returns false.
    pub fn insert_epoch(
        &mut self,
        finalized_epoch: Arc<FinalizedEpoch>,
    ) -> bool {
        if !self.epoch_number_consistent(&finalized_epoch.epoch_number) {
            return false;
        }

        self.finalized_epochs.push_back(finalized_epoch);
        true
    }
}

#[async_trait]
impl Foldable for FinalizedEpochs {
    type InitialState = Arc<RollupsInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let contract =
            RollupsFacet::new(*initial_state.dapp_contract_address, access);

        // Retrieve FinalizeEpoch events
        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context("Error querying for rollups finalized epochs")?;

        let mut finalized_epochs =
            FinalizedEpochs::new(Arc::clone(&initial_state));

        // If number of epoch finalized events is smaller than the specified
        // `inital_epoch` then no update is needed
        if epoch_finalized_events.len() < initial_state.initial_epoch.as_usize()
        {
            return Ok(finalized_epochs);
        }

        let slice =
            &epoch_finalized_events[initial_state.initial_epoch.as_usize()..];

        // For every event in `epoch_finalized_events`, considering the
        // `initial_epoch` slice, add a `FinalizedEpoch` to the list
        for (ev, meta) in slice {
            let epoch_initial_state = Arc::new(EpochInitialState {
                dapp_contract_address: Arc::clone(
                    &initial_state.dapp_contract_address,
                ),
                epoch_number: ev.epoch_number,
            });

            let inputs = EpochInputState::get_state_for_block(
                &epoch_initial_state,
                block,
                env,
            )
            .await?
            .state;

            let epoch = FinalizedEpoch {
                epoch_number: ev.epoch_number,
                hash: ev.epoch_hash.into(),
                inputs,
                finalized_block_hash: meta.block_hash,
                finalized_block_number: meta.block_number,
            };

            let inserted = finalized_epochs.insert_epoch(Arc::new(epoch));
            assert!(inserted);
        }

        Ok(finalized_epochs)
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address =
            &previous_state.rollups_initial_state.dapp_contract_address;

        // Check if there was (possibly) some log emited on this block.
        // As finalized epochs' inputs will not change, we can return early
        // without querying the input StateFold.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &previous_state.next_epoch(),
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &FinalizeEpochFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = RollupsFacet::new(**dapp_contract_address, access);

        // Retrieve finalized epoch events
        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context("Error querying for rollups finalized epochs")?;

        // Clone previous finalized epochs to the current list
        let mut finalized_epochs = previous_state.clone();

        // For every event create a new `FinalizedEpoch` and add it
        // to the list
        for (ev, meta) in epoch_finalized_events {
            if ev.epoch_number < finalized_epochs.next_epoch() {
                continue;
            }

            let epoch_initial_state = Arc::new(EpochInitialState {
                dapp_contract_address: Arc::clone(&dapp_contract_address),
                epoch_number: ev.epoch_number,
            });

            let inputs = EpochInputState::get_state_for_block(
                &epoch_initial_state,
                block,
                env,
            )
            .await?
            .state;

            let epoch = Arc::new(FinalizedEpoch {
                epoch_number: ev.epoch_number,
                hash: ev.epoch_hash.into(),
                inputs,
                finalized_block_hash: meta.block_hash,
                finalized_block_number: meta.block_number,
            });

            let inserted = finalized_epochs.insert_epoch(epoch);
            assert!(inserted);
        }

        Ok(finalized_epochs)
    }
}
