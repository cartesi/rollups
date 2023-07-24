// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum GraphQLServerError {
    #[snafu(display("health check error"))]
    HealthCheckError {
        source: http_health_check::HealthCheckError,
    },

    #[snafu(display("server error"))]
    ServerError { source: std::io::Error },
}
