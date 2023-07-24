// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use clap::Parser;
use rollups_data::{RepositoryCLIConfig, RepositoryConfig};

#[derive(Debug)]
pub struct GraphQLConfig {
    pub graphql_host: String,
    pub graphql_port: u16,
    pub repository_config: RepositoryConfig,
    pub healthcheck_port: u16,
}

#[derive(Parser)]
pub struct CLIConfig {
    #[arg(long, env, default_value = "127.0.0.1")]
    pub graphql_host: String,

    #[arg(long, env, default_value_t = 4000)]
    pub graphql_port: u16,

    #[command(flatten)]
    repository_config: RepositoryCLIConfig,

    /// Port of health check
    #[arg(long, env = "GRAPHQL_HEALTHCHECK_PORT", default_value_t = 8080)]
    pub healthcheck_port: u16,
}

impl From<CLIConfig> for GraphQLConfig {
    fn from(cli_config: CLIConfig) -> Self {
        Self {
            graphql_host: cli_config.graphql_host,
            graphql_port: cli_config.graphql_port,
            repository_config: cli_config.repository_config.into(),
            healthcheck_port: cli_config.healthcheck_port,
        }
    }
}
