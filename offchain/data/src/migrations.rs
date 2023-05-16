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
