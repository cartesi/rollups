use super::error;
use error::*;

use super::fold::types::*;
use super::{EpochStatus, MachineInterface};
use async_trait::async_trait;
use ethers::types::{H256, U256};
use im::Vector;

// pub mod versioning_proto {
//     tonic::include_proto!("versioning");
// }

pub mod cartesi_machine_proto {
    tonic::include_proto!("cartesi_machine");
}

// pub mod rollup_proto {
//     tonic::include_proto!("cartesi_rollup_machine_manager");
// }

pub struct MachineManager {}

#[async_trait]
impl MachineInterface for MachineManager {
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
