use offchain_core::ethers;

use super::error;
use error::*;

use super::fold::types::*;
use super::{EpochStatus, MachineInterface};
use async_trait::async_trait;
use im::Vector;
use snafu::ResultExt;
use std::convert::TryInto;
use tokio::sync::Mutex;

use ethers::types::{H256, U256};
use ethers::utils::keccak256;

use tonic::transport::Channel;

use cartesi_machine::{
    machine_request::MachineOneof, ConcurrencyConfig, DhdRuntimeConfig,
    MachineRequest, MachineRuntimeConfig, Void,
};
use cartesi_rollup_machine_manager::rollup_machine_manager_client::RollupMachineManagerClient;
use cartesi_rollup_machine_manager::{
    AdvanceStateRequest, CyclesConfig, DeadlineConfig, FinishEpochRequest,
    GetEpochStatusRequest, GetEpochStatusResponse, GetSessionStatusRequest,
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
    active_epoch_index: u64,
    server_deadline: DeadlineConfig,
    server_cycles: CyclesConfig,
}

impl Config {
    // TODO
    pub fn new_with_default(endpoint: String, session_id: String) -> Self {
        let machine = MachineRequest {
            runtime: Some(MachineRuntimeConfig {
                dhd: Some(DhdRuntimeConfig {
                    source_address: "".to_owned(),
                }),
                concurrency: Some(ConcurrencyConfig {
                    update_merkle_tree: 1000 * 60 * 2,
                }),
            }),

            machine_oneof: Some(MachineOneof::Directory(
                "machine_config_directory".to_owned(),
            )),
        };

        let server_deadline = DeadlineConfig {
            checkin: 1000 * 5,
            update_merkle_tree: 1000 * 60 * 2,
            advance_state: 1000 * 60 * 3,
            advance_state_increment: 1000 * 10,
            inspect_state: 0,           // TODO
            inspect_state_increment: 0, // TODO
            machine: 1000 * 60,
            store: 1000 * 60 * 3,
            fast: 1000 * 5,
        };

        let server_cycles = CyclesConfig {
            max_advance_state: u64::MAX >> 2,
            advance_state_increment: 1 << 22,
            max_inspect_state: 0,       // TODO
            inspect_state_increment: 0, // TODO
        };

        Self {
            endpoint,
            session_id,
            storage_directory: "default_storage_directory".to_owned(), // TODO
            active_epoch_index: 0,
            machine,
            server_cycles,
            server_deadline,
        }
    }
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
                    machine: Some(config.machine),
                    active_epoch_index: config.active_epoch_index,
                    server_cycles: Some(config.server_cycles),
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
            let advance_state_request =
                tonic::Request::new(AdvanceStateRequest {
                    session_id: self.session_id.clone(),
                    active_epoch_index: epoch_number.as_u64(),
                    current_input_index: first_input_index.as_u64() + i as u64,
                    input_metadata: input.get_metadata(),
                    input_payload: (*input.payload).clone(),
                });

            let _advance_response = client
                .advance_state(advance_state_request)
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
            .context(TonicStatusError)?
            .into_inner();

        let vouchers_metadata_hash = epoch_response
            .most_recent_vouchers_epoch_root_hash
            .expect("Machine Manager should return most_recent_vouchers_epoch_root_hash")
            .data;

        let notices_metadata_hash = epoch_response
            .most_recent_notices_epoch_root_hash
            .expect("Machine Manager should return most_recent_notices_epoch_root_hash")
            .data;

        let machine_state_hash = epoch_response
            .most_recent_machine_hash
            .expect("Machine Manager should return most_recent_machine_hash")
            .data;

        assert_eq!(vouchers_metadata_hash.len(), 32);
        assert_eq!(notices_metadata_hash.len(), 32);
        assert_eq!(machine_state_hash.len(), 32);

        let claim = compute_claim_hash(
            vouchers_metadata_hash.as_slice().try_into().unwrap(),
            notices_metadata_hash.as_slice().try_into().unwrap(),
            machine_state_hash.as_slice().try_into().unwrap(),
        );

        Ok(claim)
    }
}

fn compute_claim_hash(
    machine_state_hash: [u8; 32],
    vouchers_metadata_hash: [u8; 32],
    notices_metadata_hash: [u8; 32],
) -> H256 {
    let concat = [
        machine_state_hash,
        vouchers_metadata_hash,
        notices_metadata_hash,
    ]
    .concat();

    keccak256(&concat).into()
}

#[cfg(test)]
mod tests {
    use ethers::types::H256;
    use std::str::FromStr;

    use super::compute_claim_hash;

    #[test]
    fn test_claim_hash() {
        let hash: H256 = H256::from_str("0x973ec1026786d31f9980a949b9fc89726278ea9306aa6e15602ecd43f5174b94").unwrap();
        let claim = compute_claim_hash(hash.into(), hash.into(), hash.into());
        assert_eq!(
            H256::from_str("0xb19b8a98b4dc1a45afadecf00e4482b06e071f40409c866fa32b2e60f5cb3c45").unwrap(),
            claim
        );
    }
}
