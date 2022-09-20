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

pub mod config;

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use std::net::SocketAddr;

use config::HealthCheckConfig;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn start_health_check(config: HealthCheckConfig) -> Result<()> {
    tracing::trace!(?config, "starting health-check server");

    let ip = config
        .health_check_address
        .parse()
        .context("could not parse host address")?;
    let addr = SocketAddr::new(ip, config.health_check_port);
    let app = Router::new().route("/healthz", get(|| async { "" }));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("failed to start health-check server")
}
