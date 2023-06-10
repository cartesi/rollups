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

use config::DispatcherConfig;
pub use error::DispatcherError;
use snafu::ResultExt;

pub mod config;
pub mod dispatcher;
pub mod http_health;
pub mod machine;
pub mod sender;

mod auth;
mod drivers;
mod error;
mod setup;
mod signer;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: DispatcherConfig) -> Result<(), DispatcherError> {
    // tracing::info!(?config, "starting indexer");

    // let health_handle = http_health_check::start(config.health_check_config);
    // let indexer_handle = indexer::Indexer::start(config.indexer_config);

    // tokio::select! {
    //     ret = health_handle => {
    //         ret.context(error::HealthCheckSnafu)
    //     }
    //     ret = indexer_handle => {
    //         ret
    //     }
    // }

    let hc_config = config.hc_config.clone();

    let health_handle = http_health::start_health_check(
        hc_config.host_address.as_ref(),
        hc_config.port,
    );

    let dispatcher_handle = dispatcher::start(config);

    tokio::select! {
        ret = health_handle => {
            ret.context(error::HealthCheckSnafu)
        }

        ret = dispatcher_handle => {
            ret
        }
    }
}
