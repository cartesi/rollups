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

use error::InspectError;
use snafu::ResultExt;

pub use config::Config;
pub use inspect::InspectClient;

pub mod config;
mod error;
pub mod grpc;
pub mod inspect;
pub mod server;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: Config) -> Result<(), InspectError> {
    log::info!("starting inspect server with {:?}", config);
    let inspect_client = InspectClient::new(&config.inspect_server_config);
    let inspect_server =
        server::create(&config.inspect_server_config, inspect_client)
            .context(error::ServerSnafu)?;

    if config.health_check_config.healthcheck_disabled.is_none() {
        let health_handle = http_health_check::start(
            config.health_check_config.healthcheck_port,
        );

        tokio::select! {
            ret = health_handle => {
                ret.context(error::HealthCheckSnafu)
            }
            ret = inspect_server => {
                ret.context(error::ServerSnafu)
            }
        }
    } else {
        inspect_server.await.context(error::ServerSnafu)
    }
}
