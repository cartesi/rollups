// Copyright 2021 Cartesi Pte. Ltd.
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

pub mod errors;
pub mod model;
mod rollup_server;

use crate::config::Config;
use crate::controller::Controller;

/// Setup the HTTP server that receives requests from the DApp backend
pub async fn start_services(config: &Config, controller: Controller) -> std::io::Result<()> {
    rollup_server::start_service(config, controller.clone()).await
}
