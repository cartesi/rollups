// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use diesel::{pg::PgConnection, Connection};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use snafu::{ResultExt, Snafu};
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Snafu)]
pub enum MigrationError {
    #[snafu(display("connection error"))]
    ConnectionError { source: diesel::ConnectionError },

    #[snafu(display("migration error"))]
    RunMigrationError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub fn run_migrations(postgres_endpoint: &str) -> Result<(), MigrationError> {
    tracing::trace!("running pending migrations");

    let mut connection =
        PgConnection::establish(postgres_endpoint).context(ConnectionSnafu)?;
    let migrations = connection
        .run_pending_migrations(MIGRATIONS)
        .context(RunMigrationSnafu)?;
    for migration in migrations.iter() {
        tracing::trace!("runned migration {}", migration);
    }

    Ok(())
}
