// Copyright (C) 2022 Cartesi Pte. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pub mod input_test_data;
pub mod test_data;

use diesel::pg::PgConnection;
use diesel::{Connection, RunQueryDsl};

pub const POSTGRES_PORT: u16 = 5434;
pub const POSTGRES_HOSTNAME: &str = "127.0.0.1";
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_PASSWORD: &str = "password";
pub const POSTGRES_DB: &str = "test_indexer";
pub const PATH_TO_MIGRATION_FOLDER: &str = "../data/migrations/";

pub fn connect_to_database(
    postgres_endpoint: &str,
) -> Result<PgConnection, diesel::ConnectionError> {
    PgConnection::establish(&postgres_endpoint)
}

#[allow(dead_code)]
pub fn create_database(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<(), diesel::result::Error> {
    let endpoint = format!(
        "postgres://{}:{}@{}:{}",
        user,
        password,
        host,
        &port.to_string()
    );

    let conn = connect_to_database(&endpoint).unwrap();
    // Drop old database
    match diesel::sql_query(&format!("DROP DATABASE IF EXISTS {}", POSTGRES_DB))
        .execute(&conn)
    {
        Ok(res) => {
            println!("Database dropped, result {}", res);
        }
        Err(e) => {
            println!("Error dropping database: {}", e.to_string());
        }
    };

    // Create new database
    match diesel::sql_query(&format!("CREATE DATABASE {}", POSTGRES_DB))
        .execute(&conn)
    {
        Ok(res) => {
            println!("Database created, result {}", res);
        }
        Err(e) => {
            println!("Error creating database: {}", e.to_string());
        }
    };
    Ok(())
}

pub fn perform_diesel_setup(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
    database: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        host,
        &port.to_string(),
        database
    );

    std::process::Command::new("diesel")
        .arg(&format!("setup"))
        .arg(&format!("--database-url={}", endpoint))
        .arg(&format!("--migration-dir={}", PATH_TO_MIGRATION_FOLDER))
        .output()
        .expect("Unable to launch Cartesi machine server");

    Ok(())
}
