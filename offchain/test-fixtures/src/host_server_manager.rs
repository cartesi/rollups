// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use anyhow::{anyhow, Context};
use backoff::{future::retry, ExponentialBackoff, ExponentialBackoffBuilder};
use grpc_interfaces::cartesi_server_manager::{
    processed_input::ProcessedInputOneOf,
    server_manager_client::ServerManagerClient, EpochState,
    GetEpochStatusRequest, GetEpochStatusResponse, GetSessionStatusRequest,
};
use rollups_events::Payload;
use std::time::Duration;
use testcontainers::{
    clients::Cli, core::WaitFor, images::generic::GenericImage, Container,
};
use tokio::sync::Mutex;
use tonic::transport::Channel;

const SESSION_ID: &str = "default-session-id";
const RETRY_MAX_ELAPSED_TIME: u64 = 120;

macro_rules! grpc_call {
    ($self: ident, $method: ident, $request: expr) => {
        $self
            .client
            .lock()
            .await
            .$method($request)
            .await
            .map(|v| v.into_inner())
            .context("grpc call failed")
    };
}

pub struct HostServerManagerFixture<'d> {
    _node: Container<'d, GenericImage>,
    client: Mutex<ServerManagerClient<Channel>>,
    session_id: String,
    backoff: ExponentialBackoff,
    grpc_endpoint: String,
    http_endpoint: String,
}

impl HostServerManagerFixture<'_> {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn setup(docker: &Cli) -> HostServerManagerFixture<'_> {
        tracing::info!("setting up host-server-manager fixture");
        tracing::trace!("starting host-server-manager docker container");
        let image = GenericImage::new("cartesi/host-server-manager", "0.9.0")
            .with_wait_for(WaitFor::message_on_stderr(
                "starting in Actix runtime",
            ))
            .with_exposed_port(5001)
            .with_exposed_port(5004);
        let node = docker.run(image);
        let grpc_endpoint =
            format!("http://127.0.0.1:{}", node.get_host_port_ipv4(5001));
        let http_endpoint =
            format!("http://127.0.0.1:{}", node.get_host_port_ipv4(5004));
        tracing::trace!(grpc_endpoint, "connecting to host-server-manager");
        let client = Mutex::new(
            ServerManagerClient::connect(grpc_endpoint.clone())
                .await
                .expect("failed to connect to host server manager"),
        );
        let backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(Duration::from_secs(
                RETRY_MAX_ELAPSED_TIME,
            )))
            .build();

        HostServerManagerFixture {
            _node: node,
            client,
            session_id: SESSION_ID.to_owned(),
            backoff,
            grpc_endpoint,
            http_endpoint,
        }
    }

    pub fn grpc_endpoint(&self) -> &str {
        &self.grpc_endpoint
    }

    pub fn http_endpoint(&self) -> &str {
        &self.http_endpoint
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Wait until the session is ready
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn assert_session_ready(&self) {
        tracing::trace!("asserting whether session is ready");
        retry(self.backoff.clone(), || async {
            let request = GetSessionStatusRequest {
                session_id: self.session_id.clone(),
            };
            grpc_call!(self, get_session_status, request)?;
            Ok(())
        })
        .await
        .expect("failed to wait for session");
    }

    /// Wait until there is the required amount of processed inputs
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn assert_epoch_status(
        &self,
        epoch_index: u64,
        expected_processed: usize,
    ) -> GetEpochStatusResponse {
        tracing::trace!(
            epoch_index,
            expected_processed,
            "asserting epoch status"
        );
        retry(self.backoff.clone(), || async {
            let request = GetEpochStatusRequest {
                session_id: self.session_id.clone(),
                epoch_index,
            };
            let response = grpc_call!(self, get_epoch_status, request)?;
            if response.processed_inputs.len() != expected_processed {
                Err(anyhow!(
                    "processed_inputs_count fail got={} expected={}",
                    response.processed_inputs.len(),
                    expected_processed
                ))?;
            }
            Ok(response)
        })
        .await
        .expect("failed to wait for epoch status")
    }

    /// Wait until there is the required amount of processed inputs
    /// Then, compare the obtained output payloads with the expected ones
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn assert_epoch_status_payloads(
        &self,
        epoch_index: u64,
        expected_payloads: &[Payload],
    ) {
        tracing::trace!(
            epoch_index,
            ?expected_payloads,
            "asserting epoch status payloads"
        );
        let epoch_status = self
            .assert_epoch_status(epoch_index, expected_payloads.len())
            .await;

        assert_eq!(
            expected_payloads.len(),
            epoch_status.processed_inputs.len()
        );
        for (processed_input, expected_payload) in
            epoch_status.processed_inputs.iter().zip(expected_payloads)
        {
            let oneof =
                processed_input.processed_input_one_of.as_ref().unwrap();
            match oneof {
                ProcessedInputOneOf::AcceptedData(accepted_data) => {
                    assert_eq!(accepted_data.notices.len(), 1);
                    assert_eq!(
                        &accepted_data.notices[0].payload,
                        expected_payload.inner()
                    );
                }
                ProcessedInputOneOf::ExceptionData(_) => {
                    panic!("unexpected exception data");
                }
            }
        }
    }

    /// Wait until the given epoch is finished.
    /// Raises error if the epoch is not finished after the backoff timeout.
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn assert_epoch_finished(&self, epoch_index: u64) {
        tracing::trace!(epoch_index, "asserting epoch finished");
        retry(self.backoff.clone(), || async {
            let request = GetEpochStatusRequest {
                session_id: self.session_id.clone(),
                epoch_index,
            };
            let response = grpc_call!(self, get_epoch_status, request)?;
            if response.state() == EpochState::Active {
                Err(anyhow!("epoch {} is not finished", epoch_index))?;
            }
            Ok(())
        })
        .await
        .expect("failed to wait for epoch status")
    }
}
