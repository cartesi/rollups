use crate::error::*;
use crate::state::writer::Writer;
use snafu::ResultExt;

use crate::db::PollingPool;
use crate::grpc::state_server::{
    delegate_manager_client::DelegateManagerClient, GetStateRequest,
};

use tonic::transport::Channel;

use ethers::core::types::{Address, U256};

#[derive(Clone)]
pub struct Poller {
    client: DelegateManagerClient<Channel>,
    writer: Writer,
}

impl Poller {
    pub async fn new(
        state_server_endpoint: String,
        pool: PollingPool,
    ) -> Result<Self> {
        let client = DelegateManagerClient::connect(state_server_endpoint)
            .await
            .context(TonicTransportError)?;

        let writer = Writer { pool };

        Ok(Poller { client, writer })
    }

    pub async fn poll(
        mut self,
        inner: &(U256, Address),
        poll_time: std::time::Duration,
    ) -> Result<()> {
        let json_initial_state =
            serde_json::to_string(inner).context(SerializeError)?;
        let req = GetStateRequest {
            json_initial_state: json_initial_state.clone(),
        };

        loop {
            let response = self
                .client
                .get_state(tonic::Request::new(req.clone()))
                .await
                .context(TonicStatusError)?
                .into_inner();

            self.writer.consume(response).await?;

            tokio::time::sleep(poll_time).await;
        }
    }
}
