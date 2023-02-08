// Copyright 2023 Cartesi Pte. Ltd.
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

use diesel::{pg::PgConnection, prelude::*, sql_query};
use testcontainers::{clients::Cli, images::postgres::Postgres, RunnableImage};

const POSTGRES_PASSWORD: &'static str = "pw";

fn postgres_endpoint(port: u16) -> String {
    format!(
        "postgres://postgres:{}@localhost:{}/postgres",
        POSTGRES_PASSWORD, port
    )
}

table! {
    pg_tables (tablename) {
        tablename -> VarChar,
    }
}

#[derive(Debug, QueryableByName)]
#[diesel(table_name = pg_tables)]
pub struct PgTable {
    pub tablename: String,
}

#[test_log::test(test)]
fn run_migrations() {
    tracing::info!("setting up Postgres container");
    let docker = Cli::default();
    let image = RunnableImage::from(Postgres::default())
        .with_tag("13")
        .with_env_var(("POSTGRES_PASSWORD", POSTGRES_PASSWORD));
    let postgres = docker.run(image);
    let endpoint = postgres_endpoint(postgres.get_host_port_ipv4(5432));

    tracing::info!("running migrations");
    rollups_data::run_migrations(&endpoint).expect("failed to run migrations");

    tracing::info!("checking whether migrations run in DB");
    let mut connection = PgConnection::establish(&endpoint)
        .expect("failed to establish connection");
    let tables = sql_query("SELECT tablename FROM pg_tables;")
        .load::<PgTable>(&mut connection)
        .expect("failed to run query");

    let expected_tables =
        vec!["inputs", "vouchers", "notices", "reports", "proofs"];
    for expected in expected_tables {
        assert!(tables.iter().find(|t| t.tablename == expected).is_some());
    }
}
