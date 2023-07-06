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

use clap::Parser;
use rollups_data::{RepositoryCLIConfig, RepositoryConfig};

#[derive(Debug)]
pub struct GraphQLConfig {
    pub graphql_host: String,
    pub graphql_port: u16,
    pub repository_config: RepositoryConfig,
}

#[derive(Debug, Clone, Parser)]
pub struct GraphQLHealthCheckConfig {
    /// Enable or disable health check
    #[arg(
        long = "graphql-healthcheck-enabled",
        env = "GRAPHQL_HEALTHCHECK_ENABLED",
        default_value_t = true
    )]
    pub enabled: bool,

    /// Port of health check
    #[arg(
        long = "graphql-healthcheck-port",
        env = "GRAPHQL_HEALTHCHECK_PORT",
        default_value_t = 8080
    )]
    pub port: u16,
}

#[derive(Debug)]
pub struct Config {
    pub graphql_config: GraphQLConfig,
    pub health_check_config: GraphQLHealthCheckConfig,
}

#[derive(Parser)]
pub struct CLIConfig {
    #[arg(long, env, default_value = "127.0.0.1")]
    pub graphql_host: String,

    #[arg(long, env, default_value_t = 4000)]
    pub graphql_port: u16,

    #[command(flatten)]
    repository_config: RepositoryCLIConfig,

    #[command(flatten)]
    health_check_config: GraphQLHealthCheckConfig,
}

impl From<CLIConfig> for Config {
    fn from(cli_config: CLIConfig) -> Self {
        let graphql_config = GraphQLConfig {
            graphql_host: cli_config.graphql_host,
            graphql_port: cli_config.graphql_port,
            repository_config: cli_config.repository_config.into(),
        };

        Self {
            graphql_config,
            health_check_config: cli_config.health_check_config,
        }
    }
}
