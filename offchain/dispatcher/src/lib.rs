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

pub mod config;
pub mod dispatcher;
pub mod machine;
pub mod sender;

mod auth;
mod drivers;
mod error;
mod metrics;
mod setup;
mod signer;

use config::Config;
use error::DispatcherError;
use metrics::DispatcherMetrics;
use snafu::ResultExt;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: Config) -> Result<(), DispatcherError> {
    let metrics = DispatcherMetrics::default();
    let dispatcher_handle =
        dispatcher::start(config.dispatcher_config, metrics.clone());
    let http_server_handle =
        http_server::start(config.http_server_config, metrics.into());
    tokio::select! {
        ret = http_server_handle => {
            ret.context(error::HttpServerSnafu)
        }
        ret = dispatcher_handle => {
            ret
        }
    }
}
