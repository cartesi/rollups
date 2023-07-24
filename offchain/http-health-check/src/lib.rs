// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
