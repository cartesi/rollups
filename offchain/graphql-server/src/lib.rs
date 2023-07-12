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

use snafu::ResultExt;

pub use config::{CLIConfig, GraphQLConfig};
pub use error::GraphQLServerError;
pub use http::start_service;
pub use schema::Context;

pub mod config;
mod error;
pub mod http;
pub mod schema;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: GraphQLConfig) -> Result<(), GraphQLServerError> {
    tracing::info!(?config, "starting graphql http service");

    let repository = rollups_data::Repository::new(config.repository_config)
        .expect("failed to connect to database");
    let context = Context::new(repository);
    let service_handler =
        start_service(&config.graphql_host, config.graphql_port, context)
            .expect("failed to create server");

    let health_handle = http_health_check::start(config.healthcheck_port);

    tokio::select! {
        ret = health_handle => {
            ret.context(error::HealthCheckSnafu)
        }
        ret = service_handler => {
            ret.context(error::ServerSnafu)
        }
    }
}
