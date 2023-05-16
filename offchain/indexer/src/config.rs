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

pub use http_health_check::HealthCheckConfig;
pub use rollups_data::{RepositoryCLIConfig, RepositoryConfig};
pub use rollups_events::{
    BrokerCLIConfig, BrokerConfig, DAppMetadata, DAppMetadataCLIConfig,
};

#[derive(Debug)]
pub struct IndexerConfig {
    pub repository_config: RepositoryConfig,
    pub dapp_metadata: DAppMetadata,
    pub broker_config: BrokerConfig,
}

#[derive(Debug)]
pub struct Config {
    pub indexer_config: IndexerConfig,
    pub health_check_config: HealthCheckConfig,
}

#[derive(Parser)]
pub struct CLIConfig {
    #[command(flatten)]
    repository_config: RepositoryCLIConfig,

    #[command(flatten)]
    dapp_metadata_config: DAppMetadataCLIConfig,

    #[command(flatten)]
    broker_config: BrokerCLIConfig,

    #[command(flatten)]
    health_check_config: HealthCheckConfig,
}

impl From<CLIConfig> for Config {
    fn from(cli_config: CLIConfig) -> Self {
        let indexer_config = IndexerConfig {
            repository_config: cli_config.repository_config.into(),
            dapp_metadata: cli_config.dapp_metadata_config.into(),
            broker_config: cli_config.broker_config.into(),
        };
        Self {
            indexer_config,
            health_check_config: cli_config.health_check_config,
        }
    }
}
