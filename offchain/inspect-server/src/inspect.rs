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

use snafu::Snafu;
use tokio::sync::{mpsc, oneshot};
use tonic::Request;
use uuid::Uuid;

use crate::config::Config;
use crate::grpc::server_manager::{
    server_manager_client::ServerManagerClient, InspectStateRequest,
};

pub use crate::grpc::server_manager::{
    CompletionStatus, InspectStateResponse, Report,
};

#[derive(Debug, Snafu)]
pub enum InspectError {
    #[snafu(display("Failed to connect to server manager: {}", message))]
    FailedToConnect { message: String },
    #[snafu(display("Failed to inspect state: {}", message))]
    InspectFailed { message: String },
}

#[derive(Clone)]
pub struct InspectClient {
    inspect_tx: mpsc::Sender<InspectRequest>,
}

/// The inspect client is a wrapper that just sends the inspect requests to another thread and
/// waits for the result. The actual request to the server manager is done by the handle_inspect
/// function.
impl InspectClient {
    pub fn new(config: &Config) -> Self {
        let (inspect_tx, inspect_rx) = mpsc::channel(config.queue_size);
        let address = config.server_manager_address.clone();
        let session_id = config.session_id.clone();
        tokio::spawn(handle_inspect(address, session_id, inspect_rx));
        Self { inspect_tx }
    }

    pub async fn inspect(
        &self,
        payload: Vec<u8>,
    ) -> Result<InspectStateResponse, InspectError> {
        let (response_tx, response_rx) = oneshot::channel();
        let request = InspectRequest {
            payload,
            response_tx,
        };
        if let Err(e) = self.inspect_tx.try_send(request) {
            return Err(InspectError::InspectFailed {
                message: e.to_string(),
            });
        } else {
            log::debug!("inspect request added to the queue");
        }
        response_rx.await.expect("handle_inspect never fails")
    }
}

struct InspectRequest {
    payload: Vec<u8>,
    response_tx: oneshot::Sender<Result<InspectStateResponse, InspectError>>,
}

fn respond(
    response_tx: oneshot::Sender<Result<InspectStateResponse, InspectError>>,
    response: Result<InspectStateResponse, InspectError>,
) {
    if let Err(_) = response_tx.send(response) {
        log::warn!("failed to respond inspect request (client dropped)");
    }
}

/// Loop that answers requests comming from inspect_rx.
async fn handle_inspect(
    address: String,
    session_id: String,
    mut inspect_rx: mpsc::Receiver<InspectRequest>,
) {
    let endpoint = format!("http://{}", address);
    while let Some(request) = inspect_rx.recv().await {
        match ServerManagerClient::connect(endpoint.clone()).await {
            Err(e) => {
                respond(
                    request.response_tx,
                    Err(InspectError::FailedToConnect {
                        message: e.to_string(),
                    }),
                );
            }
            Ok(mut client) => {
                let request_id = Uuid::new_v4().to_string();
                let grpc_request = InspectStateRequest {
                    session_id: session_id.clone(),
                    query_payload: request.payload,
                };

                log::debug!(
                    "calling grpc inspect_state request={:?} request_id={}",
                    grpc_request,
                    request_id
                );
                let mut grpc_request = Request::new(grpc_request);
                grpc_request
                    .metadata_mut()
                    .insert("request-id", request_id.parse().unwrap());
                let grpc_response = client.inspect_state(grpc_request).await;

                log::debug!("got grpc response from inspect_state response={:?} request_id={}", grpc_response, request_id);

                let response = grpc_response
                    .map(|result| result.into_inner())
                    .map_err(|e| InspectError::InspectFailed {
                        message: e.message().to_string(),
                    });
                respond(request.response_tx, response);
            }
        }
    }
}
