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

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;

#[derive(Debug, Clone, Parser)]
#[command(name = "http-health-check")]
pub struct HealthCheckConfig {
    /// Host address of health check
    #[arg(long, env, default_value = "0.0.0.0")]
    pub health_check_address: String,

    /// Port of health check
    #[arg(long, env, default_value = "8080")]
    pub health_check_port: u16,
}

#[tracing::instrument(level = "trace", skip_all)]
pub async fn start(config: HealthCheckConfig) -> Result<()> {
    tracing::trace!(?config, "starting health-check server");

    let ip = config
        .health_check_address
        .parse()
        .context("could not parse host address")?;
    let addr = SocketAddr::new(ip, config.health_check_port);
    let app = Router::new().route("/healthz", get(|| async { "" }));
    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    tracing::trace!(address = ?server.local_addr(), "http healthcheck address bound");

    server.await.context("failed to start health-check server")
}
