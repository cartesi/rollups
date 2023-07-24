// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
