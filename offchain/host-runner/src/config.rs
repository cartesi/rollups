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
}
