use super::error;

use super::fold::types::*;
use async_trait::async_trait;
use error::*;
use ethers::types::{H256, U256};
use im::Vector;

pub struct EpochStatus {
    pub epoch_number: U256,
    pub processed_input_count: usize,
    pub pending_input_count: usize,
    pub is_active: bool,
}

// TODO: what happens with skipped inputs?
#[async_trait]
pub trait MachineInterface {
    async fn get_current_epoch_status(&self) -> Result<EpochStatus>;

    async fn enqueue_inputs(
        &self,
        epoch_number: U256,
        first_input_index: U256,
        inputs: Vector<Input>,
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

pub struct MockMachine {}

#[async_trait]
impl MachineInterface for MockMachine {
    async fn get_current_epoch_status(&self) -> Result<EpochStatus> {
        todo!()
    }

    async fn enqueue_inputs(
        &self,
        epoch_number: U256,
        first_input_index: U256,
        inputs: Vector<Input>,
    ) -> Result<()> {
        todo!()
    }

    async fn finish_epoch(
        &self,
        epoch_number: U256,
        input_count: U256,
    ) -> Result<()> {
        todo!()
    }

    async fn get_epoch_claim(&self, epoch_number: U256) -> Result<H256> {
        todo!()
    }
}
