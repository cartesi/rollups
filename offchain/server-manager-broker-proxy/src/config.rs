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

use clap::Parser;
use snafu::{ResultExt, Snafu};
use std::time::Duration;

use crate::broker::config::{BrokerCLIConfig, BrokerConfig, BrokerConfigError};
use crate::http_health::config::HealthCheckConfig;
use crate::server_manager::config::{
    ServerManagerCLIConfig, ServerManagerConfig,
};

#[derive(Debug)]
pub struct Config {
    pub proxy_config: ProxyConfig,
    pub health_check_config: HealthCheckConfig,
}

#[derive(Debug)]
pub struct ProxyConfig {
    pub server_manager_config: ServerManagerConfig,
    pub broker_config: BrokerConfig,
    pub backoff_max_elapsed_duration: Duration,
}

impl Config {
    pub fn parse() -> Result<Self, ConfigError> {
        let cli_config = CLIConfig::parse();
        let broker_config =
            BrokerConfig::parse_from_cli(cli_config.broker_cli_config)
                .context(BrokerConfigSnafu)?;
        let server_manager_config =
            ServerManagerConfig::parse_from_cli(cli_config.sm_cli_config);
        let backoff_max_elapsed_duration =
            Duration::from_millis(cli_config.backoff_max_elapsed_duration);
        let proxy_config = ProxyConfig {
            server_manager_config,
            broker_config,
            backoff_max_elapsed_duration,
        };
        Ok(Self {
            proxy_config,
            health_check_config: cli_config.health_check_config,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("error in broker configuration"))]
    BrokerConfigError { source: BrokerConfigError },
}

#[derive(Parser, Debug)]
struct CLIConfig {
    #[command(flatten)]
    sm_cli_config: ServerManagerCLIConfig,

    #[command(flatten)]
    broker_cli_config: BrokerCLIConfig,

    #[command(flatten)]
    health_check_config: HealthCheckConfig,

    /// The max elapsed time for backoff in ms
    #[arg(long, env, default_value = "120000")]
    backoff_max_elapsed_duration: u64,
}
