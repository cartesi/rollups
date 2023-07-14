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

pub const GRPC_SERVER_MANAGER_PORT: u16 = 50001;
pub const HTTP_INSPECT_PORT: u16 = 50002;
pub const HTTP_ROLLUP_SERVER_PORT: u16 = 50004;
pub const FINISH_TIMEOUT: u64 = 100;

pub fn get_grpc_server_manager_address() -> String {
    format!("http://127.0.0.1:{}", GRPC_SERVER_MANAGER_PORT)
}

pub fn get_http_inspect_address() -> String {
    format!("http://127.0.0.1:{}", HTTP_INSPECT_PORT)
}

pub fn get_http_rollup_server_address() -> String {
    format!("http://127.0.0.1:{}", HTTP_ROLLUP_SERVER_PORT)
}

pub fn get_host_runner_path() -> String {
    std::env::var("CARTESI_HOST_RUNNER_PATH")
        .unwrap_or(String::from("../target/debug/cartesi-rollups-host-runner"))
}

pub fn get_test_verbose() -> bool {
    std::env::var("CARTESI_TEST_VERBOSE").is_ok()
}
