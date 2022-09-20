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
use tonic::transport::Channel;

use rollups_events::rollups_inputs::InputMetadata;

use cartesi_machine::{
    ConcurrencyConfig, DhdRuntimeConfig, MachineRuntimeConfig, Void,
};
use cartesi_server_manager::server_manager_client::ServerManagerClient;
use cartesi_server_manager::{
    Address, AdvanceStateRequest, CyclesConfig, DeadlineConfig,
    EndSessionRequest, FinishEpochRequest, GetEpochStatusRequest,
    InputMetadata as MMInputMetadata, StartSessionRequest,
};

use claim::compute_claim_hash;
use config::ServerManagerConfig;

mod claim;
pub mod config;
mod versioning {
    tonic::include_proto!("versioning");
}
mod cartesi_machine {
    tonic::include_proto!("cartesi_machine");
}
mod cartesi_server_manager {
    tonic::include_proto!("cartesi_server_manager");
}

#[derive(Debug, Snafu)]
pub enum ServerManagerError {
    #[snafu(display("failed to connect to server-manager"))]
    ConnectionError { source: tonic::transport::Error },

    #[snafu(display("{} call failed", method))]
    MethodCallError {
        method: String,
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
    pub async fn start_session(&mut self) -> Result<()> {
        tracing::trace!("starting server-manager session");

        // If session exists, delete it before creating new one
        let response = retry(self.backoff.clone(), || async {
            tracing::trace!("calling grpc get_status");

            let response = self
                .client
                .clone()
                .get_status(Void {})
                .await
                .context(MethodCallSnafu {
                    method: "get_status",
                })?
                .into_inner();

            tracing::trace!(?response, "got grpc response");
            Ok(response)
        })
        .await?;

        let session_exists =
            response.session_id.contains(&self.config.session_id);
        if session_exists {
            retry(self.backoff.clone(), || async {
                let request = EndSessionRequest {
                    session_id: self.config.session_id.clone(),
                };

                tracing::trace!(?request, "calling grpc end_session",);

                let response =
                    self.client.clone().end_session(request).await.context(
                        MethodCallSnafu {
                            method: "end_session",
                        },
                    )?;

                tracing::trace!(?response, "got grpc response");

                Ok(())
            })
            .await?;
        }

        retry(self.backoff.clone(), || async {
            let machine_directory = "/opt/cartesi/share/dapp-bin".to_owned();

            let runtime = Some(MachineRuntimeConfig {
                dhd: Some(DhdRuntimeConfig {
                    source_address: "".to_owned(),
                }),
                concurrency: Some(ConcurrencyConfig {
                    update_merkle_tree: 0,
                }),
            });

            let active_epoch_index = 0;

            let server_deadline = Some(DeadlineConfig {
                checkin: 1000 * 5,
                advance_state: 1000 * 60 * 3,
                advance_state_increment: 1000 * 10,
                inspect_state: 1000 * 60 * 3,
                inspect_state_increment: 1000 * 10,
                machine: 1000 * 60 * 5,
                store: 1000 * 60 * 3,
                fast: 1000 * 5,
            });

            let server_cycles = Some(CyclesConfig {
                max_advance_state: u64::MAX >> 2,
                advance_state_increment: 1 << 22,
                max_inspect_state: u64::MAX >> 2,
                inspect_state_increment: 1 << 22,
            });

            let new_session_request = StartSessionRequest {
                session_id: self.config.session_id.clone(),
                machine_directory,
                runtime,
                active_epoch_index,
                server_cycles,
                server_deadline,
            };

            tracing::trace!(
                request = ?new_session_request,
                "calling grpc start_session"
            );

            let response = self
                .client
                .clone()
                .start_session(new_session_request)
                .await
                .context(MethodCallSnafu {
                    method: "start_session",
                })?;

            tracing::trace!(?response, "got grpc response");
            Ok(())
        })
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn advance_state(
        &mut self,
        input_metadata: InputMetadata,
        input_payload: Vec<u8>,
    ) -> Result<()> {
        tracing::trace!("sending advance-state input to server-manager");

        retry(self.backoff.clone(), || async {
            let metadata = MMInputMetadata {
                msg_sender: Some(Address {
                    data: input_metadata.msg_sender.into(),
                }),
                block_number: input_metadata.block_number,
                timestamp: input_metadata.timestamp,
                epoch_index: input_metadata.epoch_index,
                input_index: input_metadata.input_index,
            };
            let request = AdvanceStateRequest {
                session_id: self.config.session_id.to_owned(),
                active_epoch_index: input_metadata.epoch_index,
                current_input_index: input_metadata.input_index,
                input_metadata: Some(metadata),
                input_payload: input_payload.clone(),
            };

            tracing::trace!(?request, "calling grpc advance_state",);

            let response =
                self.client.clone().advance_state(request).await.context(
                    MethodCallSnafu {
                        method: "advance_state",
                    },
                )?;

            tracing::trace!(?response, "got grpc response");

            Ok(())
        })
        .await?;

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
            let response = retry(self.backoff.clone(), || async {
                let request = GetEpochStatusRequest {
                    session_id: self.config.session_id.to_owned(),
                    epoch_index,
                };

                tracing::trace!(?request, "calling grpc get_epoch_status",);

                let response = self
                    .client
                    .clone()
                    .get_epoch_status(request)
                    .await
                    .context(MethodCallSnafu {
                        method: "get_epoch_status",
                    })?
                    .into_inner();

                tracing::trace!(?response, "got grpc response");

                Ok(response)
            })
            .await?;

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
    ) -> Result<()> {
        tracing::info!(active_epoch_index, "sending finish epoch");

        // Wait for pending inputs before sending a finish request
        let processed_input_count =
            self.wait_for_pending_inputs(active_epoch_index).await?;

        retry(self.backoff.clone(), || async {
            let request = FinishEpochRequest {
                session_id: self.config.session_id.to_owned(),
                active_epoch_index,
                processed_input_count,
                storage_directory: "".to_owned(),
            };

            tracing::trace!(?request, "calling grpc finish_epoch");

            let response =
                self.client.clone().finish_epoch(request).await.context(
                    MethodCallSnafu {
                        method: "finish_epoch",
                    },
                )?;

            tracing::trace!(?response, "got grpc response");

            Ok(())
        })
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn get_epoch_claim(
        &mut self,
        epoch_index: u64,
    ) -> Result<[u8; 32]> {
        tracing::trace!(epoch_index, "getting epoch claim");

        let response = retry(self.backoff.clone(), || async {
            let request = GetEpochStatusRequest {
                session_id: self.config.session_id.to_owned(),
                epoch_index,
            };

            tracing::trace!(?request, "calling grpc get_epoch_status");

            let response = self
                .client
                .clone()
                .get_epoch_status(request)
                .await
                .context(MethodCallSnafu {
                    method: "get_epoch_status",
                })?
                .into_inner();

            tracing::trace!(?response, "got grpc response");

            Ok(response)
        })
        .await?;

        let vouchers_metadata_hash = response
            .most_recent_vouchers_epoch_root_hash
            .expect("server-manager should return most_recent_vouchers_epoch_root_hash")
            .data;
        let notices_metadata_hash = response
            .most_recent_notices_epoch_root_hash
            .expect("server-manager should return most_recent_notices_epoch_root_hash")
            .data;
        let machine_state_hash = response
            .most_recent_machine_hash
            .expect("server-manager should return most_recent_machine_hash")
            .data;
        assert_eq!(vouchers_metadata_hash.len(), 32);
        assert_eq!(notices_metadata_hash.len(), 32);
        assert_eq!(machine_state_hash.len(), 32);

        let hash = compute_claim_hash(
            &vouchers_metadata_hash,
            &notices_metadata_hash,
            &machine_state_hash,
        );
        tracing::trace!(claim = hex::encode(hash), "computed claim hash");

        Ok(hash)
    }
}
