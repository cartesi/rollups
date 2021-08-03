use super::error;
use error::*;

use super::fold::types::*;
use super::{EpochStatus, MachineInterface};
use async_trait::async_trait;
use ethers::types::{H256, U256};
use im::Vector;
use snafu::ResultExt;
use tokio::sync::Mutex;

use tonic::transport::Channel;

use cartesi_machine::{MachineRequest, Void};
use cartesi_rollup_machine_manager::rollup_machine_manager_client::RollupMachineManagerClient;
use cartesi_rollup_machine_manager::{
    DeadlineConfig, EnqueueInputRequest, FinishEpochRequest,
    GetEpochStatusRequest, GetEpochStatusResponse, GetSessionStatusRequest,
    PayloadAndMetadata, PayloadAndMetadataArray, ProcessedInput,
    StartSessionRequest,
};

pub mod versioning {
    tonic::include_proto!("versioning");
}

pub mod cartesi_machine {
    tonic::include_proto!("cartesi_machine");
}

pub mod cartesi_rollup_machine_manager {
    tonic::include_proto!("cartesi_rollup_machine_manager");
}

impl From<GetEpochStatusResponse> for EpochStatus {
    fn from(status: GetEpochStatusResponse) -> Self {
        Self {
            epoch_number: status.epoch_index.into(),
            processed_input_count: status.processed_inputs.len(),
            pending_input_count: status.pending_input_count as usize,
            is_active: status.state == 0,
        }
    }
}

pub struct Config {
    endpoint: String,
    session_id: String,
    storage_directory: String,
    machine: MachineRequest,
    max_cycles_per_input: u64,
    cycles_per_input_chunk: u64,
    input_description: PayloadAndMetadata,
    outputs_description: PayloadAndMetadataArray,
    messages_description: PayloadAndMetadataArray,
    server_deadline: DeadlineConfig,
}

pub struct MachineManager {
    session_id: String,
    storage_directory: String,
    client: Mutex<RollupMachineManagerClient<Channel>>,
}

impl MachineManager {
    pub async fn new(config: Config) -> Result<Self> {
        let mut client = RollupMachineManagerClient::connect(config.endpoint)
            .await
            .context(TonicTransportError)?;

        let get_status_request = tonic::Request::new(Void {});
        let status_response = client
            .get_status(get_status_request)
            .await
            .context(TonicStatusError)?;

        let session_exists = status_response
            .into_inner()
            .session_id
            .contains(&config.session_id);

        if !session_exists {
            let new_session_request =
                tonic::Request::new(StartSessionRequest {
                    session_id: config.session_id.clone(),
                    active_epoch_index: 0,
                    machine: Some(config.machine), // TODO: Why optionals?
                    max_cycles_per_input: config.max_cycles_per_input,
                    cycles_per_input_chunk: config.cycles_per_input_chunk,
                    input_description: Some(config.input_description),
                    outputs_description: Some(config.outputs_description),
                    messages_description: Some(config.messages_description),
                    server_deadline: Some(config.server_deadline),
                });
            let _response = client
                .start_session(new_session_request)
                .await
                .context(TonicStatusError)?;
        }

        Ok(Self {
            session_id: config.session_id,
            storage_directory: config.storage_directory,
            client: Mutex::new(client),
        })
    }
}

#[async_trait]
impl MachineInterface for MachineManager {
    async fn get_current_epoch_status(&self) -> Result<EpochStatus> {
        let mut client = self.client.lock().await;

        // Get session status
        let get_session_request =
            tonic::Request::new(GetSessionStatusRequest {
                session_id: self.session_id.clone(),
            });

        let session_response = client
            .get_session_status(get_session_request)
            .await
            .context(TonicStatusError)?;

        // Get epoch status
        let get_epoch_request = tonic::Request::new(GetEpochStatusRequest {
            session_id: self.session_id.clone(),
            epoch_index: session_response.into_inner().active_epoch_index,
        });

        let epoch_response = client
            .get_epoch_status(get_epoch_request)
            .await
            .context(TonicStatusError)?;

        Ok(epoch_response.into_inner().into())
    }

    async fn enqueue_inputs(
        &self,
        epoch_number: U256,
        first_input_index: U256,
        inputs: Vector<Input>,
    ) -> Result<()> {
        let mut client = self.client.lock().await;

        for (i, input) in inputs.iter().enumerate() {
            let enqueue_input_request =
                tonic::Request::new(EnqueueInputRequest {
                    session_id: self.session_id.clone(),
                    active_epoch_index: epoch_number.as_u64(),
                    current_input_index: first_input_index.as_u64() + i as u64,
                    input_metadata: input.get_metadata(),
                    input_payload: (*input.payload).clone(),
                });

            let _enqueue_response = client
                .enqueue_input(enqueue_input_request)
                .await
                .context(TonicStatusError)?;
        }

        Ok(())
    }

    async fn finish_epoch(
        &self,
        epoch_number: U256,
        input_count: U256,
    ) -> Result<()> {
        let mut client = self.client.lock().await;

        let finish_epoch_request = tonic::Request::new(FinishEpochRequest {
            session_id: self.session_id.clone(),
            active_epoch_index: epoch_number.as_u64(),
            processed_input_count: input_count.as_u64(),
            storage_directory: self.storage_directory.clone(),
        });

        let _finish_response = client
            .finish_epoch(finish_epoch_request)
            .await
            .context(TonicStatusError)?;

        Ok(())
    }

    async fn get_epoch_claim(&self, epoch_number: U256) -> Result<H256> {
        let mut client = self.client.lock().await;

        // Get epoch status
        let get_epoch_request = tonic::Request::new(GetEpochStatusRequest {
            session_id: self.session_id.clone(),
            epoch_index: epoch_number.as_u64(),
        });

        let epoch_response = client
            .get_epoch_status(get_epoch_request)
            .await
            .context(TonicStatusError)?;

        // Ok(())

        todo!()
    }
}

/*
// TODO: drop (async issues)
impl Drop for MachineManager {
    fn drop(&mut self) {
            let end_session_request = tonic::Request::new(EndSessionRequest {
                session_id: self.session_id,
            });

            let _response = self
                .client
                .end_session(end_session_request)
                .await
                .expect("Couldn't end session");
    }
}
*/
