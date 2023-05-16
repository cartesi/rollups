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

#[derive(Clone, Parser)]
#[command(name = "hc_config")]
#[command(about = "Configuration for rollups dispatcher health check")]
pub struct HealthCheckEnvCLIConfig {
    /// Host address of health check
    #[arg(long, env)]
    pub hc_host_address: Option<String>,

    /// Port of health check
    #[arg(long, env)]
    pub hc_port: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct HealthCheckConfig {
    pub host_address: String,
    pub port: u16,
}

const DEFAULT_HOST_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 80;

impl HealthCheckConfig {
    pub fn initialize_from_args() -> Self {
        Self::initialize(HealthCheckEnvCLIConfig::parse())
    }

    pub fn initialize(env_cli_config: HealthCheckEnvCLIConfig) -> Self {
        let host_address = env_cli_config
            .hc_host_address
            .unwrap_or(DEFAULT_HOST_ADDRESS.to_owned());

        let port = env_cli_config.hc_port.unwrap_or(DEFAULT_PORT);

        Self { host_address, port }
    }
}
