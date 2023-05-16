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

use inspect_server::{config::Config, inspect::InspectClient, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .target(env_logger::fmt::Target::Stdout)
    .init();
    let config = Config::initialize()?;
    log::info!("starting inspect server with {:?}", config);
    let inspect_client = InspectClient::new(&config);
    server::create(&config, inspect_client)?
        .await
        .map_err(|e| e.into())
}
