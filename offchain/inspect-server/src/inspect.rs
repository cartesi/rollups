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

use snafu::Snafu;

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
    address: String,
    session_id: String,
}

impl InspectClient {
    pub fn new(config: &Config) -> Self {
        Self {
            address: config.server_manager_address.clone(),
            session_id: config.session_id.clone(),
        }
    }

    pub async fn inspect(
        &self,
        payload: Vec<u8>,
    ) -> Result<InspectStateResponse, InspectError> {
        let endpoint = format!("http://{}", self.address);
        let mut client =
            ServerManagerClient::connect(endpoint).await.map_err(|e| {
                InspectError::FailedToConnect {
                    message: e.to_string(),
                }
            })?;
        let request = InspectStateRequest {
            session_id: self.session_id.clone(),
            query_payload: payload,
        };
        client
            .inspect_state(request)
            .await
            .map(|result| result.into_inner())
            .map_err(|e| InspectError::InspectFailed {
                message: e.message().to_string(),
            })
    }
}
