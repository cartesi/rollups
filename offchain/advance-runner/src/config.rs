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
use snafu::{ResultExt, Snafu};
use std::time::Duration;

use crate::server_manager::ServerManagerCLIConfig;
pub use crate::server_manager::ServerManagerConfig;
pub use crate::snapshot::config::{FSManagerConfig, SnapshotConfig};
use crate::snapshot::config::{SnapshotCLIConfig, SnapshotConfigError};
pub use http_health_check::HealthCheckConfig;
pub use rollups_events::{
    BrokerCLIConfig, BrokerConfig, DAppMetadata, DAppMetadataCLIConfig,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub advance_runner_config: AdvanceRunnerConfig,
    pub health_check_config: HealthCheckConfig,
}

#[derive(Debug, Clone)]
pub struct AdvanceRunnerConfig {
    pub server_manager_config: ServerManagerConfig,
    pub broker_config: BrokerConfig,
    pub dapp_metadata: DAppMetadata,
    pub snapshot_config: SnapshotConfig,
    pub backoff_max_elapsed_duration: Duration,
}

impl Config {
    pub fn parse() -> Result<Self, ConfigError> {
        let cli_config = CLIConfig::parse();
        let broker_config = cli_config.broker_cli_config.into();
        let dapp_metadata = cli_config.dapp_metadata_cli_config.into();
        let server_manager_config =
            ServerManagerConfig::parse_from_cli(cli_config.sm_cli_config);
        let snapshot_config =
            SnapshotConfig::parse_from_cli(cli_config.snapshot_cli_config)
                .context(SnapshotConfigSnafu)?;
        let backoff_max_elapsed_duration =
            Duration::from_millis(cli_config.backoff_max_elapsed_duration);
        let advance_runner_config = AdvanceRunnerConfig {
            server_manager_config,
            broker_config,
            dapp_metadata,
            snapshot_config,
            backoff_max_elapsed_duration,
        };
        Ok(Self {
            advance_runner_config,
            health_check_config: cli_config.health_check_config,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("error in snapshot configuration"))]
    SnapshotConfigError { source: SnapshotConfigError },
}

#[derive(Parser, Debug)]
struct CLIConfig {
    #[command(flatten)]
    sm_cli_config: ServerManagerCLIConfig,

    #[command(flatten)]
    broker_cli_config: BrokerCLIConfig,

    #[command(flatten)]
    dapp_metadata_cli_config: DAppMetadataCLIConfig,

    #[command(flatten)]
    snapshot_cli_config: SnapshotCLIConfig,

    #[command(flatten)]
    health_check_config: HealthCheckConfig,

    /// The max elapsed time for backoff in ms
    #[arg(long, env, default_value = "120000")]
    backoff_max_elapsed_duration: u64,
}
