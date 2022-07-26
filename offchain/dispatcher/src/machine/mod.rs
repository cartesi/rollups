pub mod rollup_server;
pub mod config;

use state_fold_types::ethabi::ethereum_types::{H256, U256};
use types::input::Input;

use anyhow::Result;
use async_trait::async_trait;
use im::Vector;
use std::sync::Arc;

#[derive(Debug)]
pub struct EpochStatus {
    pub epoch_number: U256,
    pub processed_input_count: usize,
    pub pending_input_count: usize,
    pub is_active: bool,
}

// TODO: what happens with skipped inputs?
#[async_trait]
pub trait MachineInterface: std::fmt::Debug {
    async fn get_current_epoch_status(&self) -> Result<EpochStatus>;

    async fn enqueue_inputs(
        &self,
        epoch_number: U256,
        first_input_index: U256,
        inputs: Vector<Arc<Input>>,
    ) -> Result<()>;

    /// Can only be called if all inputs of `epoch_number` have been processed.
    /// That is, `pending_input_count` is zero, and `processed_input_count`
    /// equals the totality of the epoch's input.
    async fn finish_epoch(
        &self,
        epoch_number: U256,
        input_count: U256,
    ) -> Result<()>;

    /// Should only be called after `finish_epoch`.
    async fn get_epoch_claim(&self, epoch_number: U256) -> Result<H256>;
}
