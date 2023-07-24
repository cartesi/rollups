// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
