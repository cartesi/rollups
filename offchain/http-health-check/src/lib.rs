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

use axum::{routing::get, Router};
use snafu::{ResultExt, Snafu};
use std::net::SocketAddr;

#[derive(Debug, Snafu)]
pub enum HealthCheckError {
    #[snafu(display("could not parse host address"))]
    ParseAddressError { source: std::net::AddrParseError },

    #[snafu(display("http health-check server error"))]
    HttpServerError { source: hyper::Error },
}

#[tracing::instrument(level = "trace", skip_all)]
pub async fn start(port: u16) -> Result<(), HealthCheckError> {
    tracing::trace!(?port, "starting health-check server on this port");

    let ip = "0.0.0.0".parse().context(ParseAddressSnafu)?;
    let addr = SocketAddr::new(ip, port);
    let app = Router::new().route("/healthz", get(|| async { "" }));
    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    tracing::trace!(address = ?server.local_addr(), "http healthcheck address bound");

    server.await.context(HttpServerSnafu)
}
