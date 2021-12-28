use crate::machine_manager::writer::Writer;

use crate::error::*;

use snafu::ResultExt;

use crate::db::PollingPool;

use crate::grpc::{
    cartesi_machine::Void,
    server_manager::{
        server_manager_client::ServerManagerClient, GetEpochStatusRequest,
        GetSessionStatusRequest,
    },
};

use tonic::transport::Channel;

#[derive(Clone)]
pub struct Poller {
    client: ServerManagerClient<Channel>,
    writer: Writer,
}

impl Poller {
    pub async fn new(
        server_manager_endpoint: String,
        pool: PollingPool,
    ) -> Result<Poller> {
        let client = ServerManagerClient::connect(server_manager_endpoint)
            .await
            .context(TonicTransportError)?;

        Ok(Poller {
            client,
            writer: Writer { pool },
        })
    }

    pub async fn poll_version(
        mut self,
        get_time: std::time::Duration,
    ) -> Result<()> {
        let request = Void {};
        loop {
            let response = self
                .client
                .get_version(tonic::Request::new(request.clone()))
                .await
                .context(TonicStatusError)?
                .into_inner();

            //TODO Transform response into the correct type
            self.writer.consume_version(&response).await?;

            tokio::time::sleep(get_time).await;
        }
    }

    pub async fn poll_status(
        mut self,
        get_time: std::time::Duration,
    ) -> Result<()> {
        let request = Void {};
        loop {
            let response = self
                .client
                .get_status(tonic::Request::new(request.clone()))
                .await
                .context(TonicStatusError)?
                .into_inner();

            //TODO Transform response into the correct type
            self.writer.consume_status(&response).await?;

            tokio::time::sleep(get_time).await;
        }
    }

    pub async fn poll_session_status(
        mut self,
        session_id: String,
        duration: std::time::Duration,
    ) -> Result<()> {
        let request = GetSessionStatusRequest {
            session_id: session_id.clone(),
        };

        loop {
            let response = self
                .client
                .get_session_status(tonic::Request::new(request.clone()))
                .await
                .context(TonicStatusError)?
                .into_inner();

            //TODO Transform response into the correct type
            self.writer.consume_session_status(&response).await?;

            let request = GetEpochStatusRequest {
                session_id: session_id.clone(),
                epoch_index: response.active_epoch_index,
            };

            let response = self
                .client
                .get_epoch_status(tonic::Request::new(request.clone()))
                .await
                .context(TonicStatusError)?
                .into_inner();

            //TODO Transform response into the correct type
            self.writer.consume_epoch_status(&response).await?;

            tokio::time::sleep(duration).await;
        }
    }
}
