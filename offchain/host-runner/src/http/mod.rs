// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pub mod errors;
pub mod model;
mod rollup_server;

use crate::config::Config;
use crate::controller::Controller;

/// Setup the HTTP server that receives requests from the DApp backend
pub async fn start_services(
    config: &Config,
    controller: Controller,
) -> std::io::Result<()> {
    rollup_server::start_service(config, controller.clone()).await
}
