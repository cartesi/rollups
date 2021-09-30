use offchain_core::ethers;
use state_fold::types::BlockState;

use crate::error::*;
use crate::fold::types::DescartesV2State;

use serde_json;
use snafu::ResultExt;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use ethers::core::types::{Address, H256, U256};

pub mod state_server {
    tonic::include_proto!("state_server");
}

use state_server::delegate_manager_client::DelegateManagerClient;
use state_server::GetStateRequest;

pub struct RollupsStateFold {
    client: Mutex<DelegateManagerClient<Channel>>,
}

impl RollupsStateFold {
    pub async fn new(endpoint: String) -> Result<Self> {
        let client = Mutex::new(
            DelegateManagerClient::connect(endpoint)
                .await
                .context(TonicTransportError)?,
        );

        Ok(Self { client })
    }

    pub async fn get_state(
        &self,
        _block_hash: &H256,
        initial_state: &(U256, Address),
    ) -> Result<DescartesV2State> {
        let mut client = self.client.lock().await;

        let req = tonic::Request::new(GetStateRequest {
            json_initial_state: serde_json::to_string(initial_state).unwrap(),
        });

        let state_json = client
            .get_state(req)
            .await
            .context(TonicStatusError)?
            .into_inner()
            .json_state;

        let state =
            serde_json::from_str(&state_json).context(DeserializeError)?;

        Ok(state)
    }
}
