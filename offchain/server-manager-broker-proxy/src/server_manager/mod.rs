// Copyright 2022 Cartesi Pte. Ltd.
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

use backoff::{future::retry, Error, ExponentialBackoff};
use snafu::{ResultExt, Snafu};
use std::path::Path;
use tonic::{transport::Channel, Request};
use uuid::Uuid;

use rollups_events::rollups_inputs::InputMetadata;

use crate::grpc::cartesi_machine::Void;
use crate::grpc::cartesi_server_manager::server_manager_client::ServerManagerClient;
use crate::grpc::cartesi_server_manager::{
    Address, AdvanceStateRequest, EndSessionRequest, FinishEpochRequest,
    GetEpochStatusRequest, GetSessionStatusRequest,
    InputMetadata as MMInputMetadata, StartSessionRequest,
};

use claim::{compute_claim_hash, CLAIM_HASH_SIZE};
use config::ServerManagerConfig;

mod claim;
pub mod config;

/// Call the grpc method passing an unique request-id and with retry
macro_rules! grpc_call {
    ($self: ident, $method: ident, $request: expr) => {
        retry($self.backoff.clone(), || async {
            let request_id = Uuid::new_v4().to_string();
            let request = $request;

            tracing::trace!(
                request_id,
                method = stringify!($method),
                ?request,
                "calling grpc"
            );

            let mut grpc_request = Request::new(request);
            grpc_request
                .metadata_mut()
                .insert("request-id", request_id.parse().unwrap());

            let response = $self.client.clone().$method(grpc_request).await;

            tracing::trace!(
                request_id,
                method = stringify!($method),
                ?response,
                "got grpc response",
            );

            response.map(|v| v.into_inner()).map_err(|status| {
                let err_type = match status.code() {
                    tonic::Code::InvalidArgument => Error::Permanent,
                    tonic::Code::NotFound => Error::Permanent,
                    tonic::Code::AlreadyExists => Error::Permanent,
                    tonic::Code::FailedPrecondition => Error::Permanent,
                    tonic::Code::OutOfRange => Error::Permanent,
                    tonic::Code::Unimplemented => Error::Permanent,
                    tonic::Code::DataLoss => Error::Permanent,
                    _ => Error::transient,
                };
                err_type(ServerManagerError::MethodCallError {
                    source: status,
                    method: stringify!($method).to_owned(),
                    request_id,
                })
            })
        })
        .await
    };
}

#[derive(Debug, Snafu)]
pub enum ServerManagerError {
    #[snafu(display("failed to connect to server-manager"))]
    ConnectionError { source: tonic::transport::Error },

    #[snafu(display(
        "failed to call {} with request-id {}",
        method,
        request_id
    ))]
    MethodCallError {
        method: String,
        request_id: String,
        source: tonic::Status,
    },

    #[snafu(display("maximum number of retries exceeded"))]
    PendingInputsExceededError {},
}

pub type Result<T> = std::result::Result<T, ServerManagerError>;

pub struct ServerManagerFacade {
    client: ServerManagerClient<Channel>,
    config: ServerManagerConfig,
    backoff: ExponentialBackoff,
}

impl ServerManagerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn new(
        config: ServerManagerConfig,
        backoff: ExponentialBackoff,
    ) -> Result<Self> {
        tracing::trace!(?config, "connecting to server manager");

        let client = retry(backoff.clone(), || async {
            ServerManagerClient::connect(config.server_manager_endpoint.clone())
                .await
                .map_err(Error::transient)
        })
        .await
        .context(ConnectionSnafu)?;

        Ok(Self {
            client,
            config,
            backoff,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn start_session(
        &mut self,
        machine_directory: &Path,
        active_epoch_index: u64,
    ) -> Result<()> {
        tracing::trace!(
            ?machine_directory,
            active_epoch_index,
            "starting server-manager session"
        );

        // If session exists, delete it before creating new one
        let response = grpc_call!(self, get_status, Void {})?;
        if response.session_id.contains(&self.config.session_id) {
            tracing::warn!("deleting previous server-manager session");
            let session_status = grpc_call!(
                self,
                get_session_status,
                GetSessionStatusRequest {
                    session_id: self.config.session_id.clone(),
                }
            )?;
            let active_epoch_index = session_status.active_epoch_index;
            let processed_input_count =
                self.wait_for_pending_inputs(active_epoch_index).await?;
            grpc_call!(
                self,
                finish_epoch,
                FinishEpochRequest {
                    session_id: self.config.session_id.clone(),
                    active_epoch_index,
                    processed_input_count,
                    storage_directory: "".to_string(),
                }
            )?;
            grpc_call!(
                self,
                end_session,
                EndSessionRequest {
                    session_id: self.config.session_id.clone(),
                }
            )?;
        }

        grpc_call!(self, start_session, {
            StartSessionRequest {
                session_id: self.config.session_id.clone(),
                machine_directory: machine_directory.to_string_lossy().into(),
                runtime: Some(self.config.runtime_config.clone()),
                active_epoch_index,
                server_cycles: Some(self.config.cycles_config.clone()),
                server_deadline: Some(self.config.deadline_config.clone()),
            }
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn advance_state(
        &mut self,
        input_metadata: InputMetadata,
        input_payload: Vec<u8>,
    ) -> Result<()> {
        tracing::trace!("sending advance-state input to server-manager");
        grpc_call!(self, advance_state, {
            let metadata = MMInputMetadata {
                msg_sender: Some(Address {
                    data: input_metadata.msg_sender.into(),
                }),
                block_number: input_metadata.block_number,
                timestamp: input_metadata.timestamp,
                epoch_index: input_metadata.epoch_index,
                input_index: input_metadata.input_index,
            };
            AdvanceStateRequest {
                session_id: self.config.session_id.to_owned(),
                active_epoch_index: input_metadata.epoch_index,
                current_input_index: input_metadata.input_index,
                input_metadata: Some(metadata),
                input_payload: input_payload.clone(),
            }
        })?;
        Ok(())
    }

    /// Wait until the server-manager processes all pending inputs
    /// Return the number of processed inputs
    #[tracing::instrument(level = "trace", skip_all)]
    async fn wait_for_pending_inputs(
        &mut self,
        epoch_index: u64,
    ) -> Result<u64> {
        tracing::trace!(epoch_index, "waiting for pending inputs");

        for _ in 0..self.config.pending_inputs_max_retries {
            let response = grpc_call!(self, get_epoch_status, {
                GetEpochStatusRequest {
                    session_id: self.config.session_id.to_owned(),
                    epoch_index,
                }
            })?;
            if response.pending_input_count > 0 {
                let duration = std::time::Duration::from_millis(
                    self.config.pending_inputs_sleep_duration,
                );
                tracing::info!(
                    "server-manager has {} pending inputs; sleeping for {} ms",
                    response.pending_input_count,
                    duration.as_millis(),
                );
                tokio::time::sleep(duration).await;
            } else {
                let processed_inputs = response.processed_inputs.len() as u64;
                return Ok(processed_inputs);
            }
        }

        tracing::warn!(
            "the number of retries while waiting for pending inputs exceeded"
        );

        Err(ServerManagerError::PendingInputsExceededError {})
    }

    /// Send a finish-epoch request to the server-manager
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn finish_epoch(
        &mut self,
        active_epoch_index: u64,
        storage_directory: &Path,
    ) -> Result<()> {
        tracing::info!(active_epoch_index, "sending finish epoch");

        // Wait for pending inputs before sending a finish request
        let processed_input_count =
            self.wait_for_pending_inputs(active_epoch_index).await?;

        grpc_call!(self, finish_epoch, {
            FinishEpochRequest {
                session_id: self.config.session_id.to_owned(),
                active_epoch_index,
                processed_input_count,
                storage_directory: storage_directory
                    .to_string_lossy()
                    .to_string(),
            }
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn get_epoch_claim(
        &mut self,
        epoch_index: u64,
    ) -> Result<[u8; 32]> {
        tracing::trace!(epoch_index, "getting epoch claim");

        let response = grpc_call!(self, get_epoch_status, {
            GetEpochStatusRequest {
                session_id: self.config.session_id.to_owned(),
                epoch_index,
            }
        })?;

        let vouchers_metadata_hash = match response
            .most_recent_vouchers_epoch_root_hash
        {
            Some(hash) => {
                if hash.data.len() != CLAIM_HASH_SIZE {
                    tracing::warn!(
                        ?hash,
                        "server-manager returned invalid most_recent_vouchers_epoch_root_hash size"
                    );
                }
                hash.data
            }
            None => {
                tracing::warn!("server-manager should return most_recent_vouchers_epoch_root_hash");
                vec![0; CLAIM_HASH_SIZE]
            }
        };

        let notices_metadata_hash = match response
            .most_recent_notices_epoch_root_hash
        {
            Some(hash) => {
                if hash.data.len() != CLAIM_HASH_SIZE {
                    tracing::warn!(
                        ?hash,
                        "server-manager returned invalid most_recent_notices_epoch_root_hash size"
                    );
                }
                hash.data
            }
            None => {
                tracing::warn!("server-manager should return most_recent_notices_epoch_root_hash");
                vec![0; CLAIM_HASH_SIZE]
            }
        };

        let machine_state_hash = match response.most_recent_machine_hash {
            Some(hash) => {
                if hash.data.len() != CLAIM_HASH_SIZE {
                    tracing::warn!(
                        ?hash,
                        "server-manager returned invalid most_recent_machine_hash size"
                    );
                }
                hash.data
            }
            None => {
                // The host-server-manager doesn't generate this hash.
                // Hence, the code shouldn't generate a warning.
                tracing::trace!(
                    "server-manager did not return most_recent_machine_hash"
                );
                vec![0; CLAIM_HASH_SIZE]
            }
        };

        let hash = compute_claim_hash(
            &vouchers_metadata_hash,
            &notices_metadata_hash,
            &machine_state_hash,
        );
        tracing::trace!(claim = hex::encode(hash), "computed claim hash");

        Ok(hash)
    }
}
