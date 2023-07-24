// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum IndexerError {
    #[snafu(display("health check error"))]
    HealthCheckError {
        source: http_health_check::HealthCheckError,
    },

    #[snafu(display("broker error"))]
    BrokerError { source: rollups_events::BrokerError },

    #[snafu(display("migrations error"))]
    MigrationsError {
        source: rollups_data::MigrationError,
    },

    #[snafu(display("repository error"))]
    RepositoryError { source: rollups_data::Error },

    #[snafu(display("join error"))]
    JoinError { source: tokio::task::JoinError },
}
