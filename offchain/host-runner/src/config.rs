// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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

    /// Port of health check
    #[arg(long, env = "HOST_RUNNER_HEALTHCHECK_PORT", default_value_t = 8080)]
    pub healthcheck_port: u16,
}
