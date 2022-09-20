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

#[derive(Parser, Debug)]
#[command(name = "server-manager")]
pub struct ServerManagerConfig {
    /// Server-manager gRPC endpoint
    #[arg(long, env, default_value = "http://127.0.0.1:5001")]
    pub server_manager_endpoint: String,

    /// Server-manager session id
    #[arg(long, env, default_value = "default_rollups_id")]
    pub session_id: String,

    /// Sleep duration while polling for pending inputs (in millis)
    #[arg(long, env, default_value = "1000")]
    pub pending_inputs_sleep_duration: u64,

    /// Max number of retries while polling for pending inputs
    #[arg(long, env, default_value = "60")]
    pub pending_inputs_max_retries: u64,
}
