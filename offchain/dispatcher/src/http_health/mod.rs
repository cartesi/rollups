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

use axum::{routing::get, Router};
use std::net::SocketAddr;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum HealthCheckError {
    #[snafu(display("could not parse host address"))]
    ParseAddressError { source: std::net::AddrParseError },

    #[snafu(display("http health-check server error"))]
    HttpServerError { source: hyper::Error },
}

pub async fn start_health_check(
    host: &str,
    port: u16,
) -> Result<(), HealthCheckError> {
    tracing::info!(
        "Starting dispatcher health check endpoint at http://{}:{}/healthz",
        host,
        port
    );

    let addr = SocketAddr::new(host.parse().context(ParseAddressSnafu)?, port);

    let app = Router::new().route("/healthz", get(|| async { "" }));

    let ret = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context(HttpServerSnafu)?;

    Ok(ret)
}
