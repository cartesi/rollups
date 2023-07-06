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

const DEFAULT_ADDRESS: &str = "0.0.0.0";

#[derive(Parser, Clone, Debug)]
pub struct Config {
    /// gRPC address of the Server Manager endpoint
    #[arg(long, env, default_value = DEFAULT_ADDRESS)]
    pub grpc_server_manager_address: String,

    /// gRPC port of the Server Manager endpoint
    #[arg(long, env, default_value = "5001")]
    pub grpc_server_manager_port: u16,

    /// HTTP address of the Inspect endpoint
    #[arg(long, env, default_value = DEFAULT_ADDRESS)]
    pub http_inspect_address: String,

    /// HTTP port of the Inspect endpoint
    #[arg(long, env, default_value = "5002")]
    pub http_inspect_port: u16,

    /// HTTP address of the Rollup Server endpoint
    #[arg(long, env, default_value = DEFAULT_ADDRESS)]
    pub http_rollup_server_address: String,

    /// HTTP port of the Rollup Server endpoint
    #[arg(long, env, default_value = "5004")]
    pub http_rollup_server_port: u16,

    /// Duration in ms for the finish request to timeout
    #[arg(long, env, default_value = "10000")]
    pub finish_timeout: u64,

    #[command(flatten)]
    pub health_check_config: HostRunnerHealthCheckConfig,
}

#[derive(Debug, Clone, Parser)]
pub struct HostRunnerHealthCheckConfig {
    /// Enable or disable health check
    #[arg(
        long = "host-runner-healthcheck-enabled",
        env = "HOST_RUNNER_HEALTHCHECK_ENABLED",
        default_value_t = true
    )]
    pub enabled: bool,

    /// Port of health check
    #[arg(
        long = "host-runner-healthcheck-port",
        env = "HOST_RUNNER_HEALTHCHECK_PORT",
        default_value_t = 8080
    )]
    pub port: u16,
}
