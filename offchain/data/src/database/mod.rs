/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */
pub mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, ManageConnection};
use diesel::{Insertable, Queryable};
use schema::{epochs, inputs, notices, proofs, reports, vouchers};
use tokio::task::JoinError;
use tracing::{error, info};

pub const CURRENT_NOTICE_EPOCH_INDEX: &str = "current_notice_epoch_index";
pub const CURRENT_VOUCHER_EPOCH_INDEX: &str = "current_voucher_epoch_index";
pub const CURRENT_REPORT_EPOCH_INDEX: &str = "current_report_epoch_index";
pub const CURRENT_INPUT_EPOCH_INDEX: &str = "current_input_epoch_index";
pub const POOL_CONNECTION_SIZE: u32 = 3;

/// Struct representing Epoch in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "epochs"]
pub struct DbEpoch {
    pub id: i32,
    pub epoch_index: i32,
}

/// Struct representing Input in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "inputs"]
pub struct DbInput {
    pub id: i32,
    pub input_index: i32,
    pub epoch_index: i32,
    pub sender: String,
    pub tx_hash: Option<String>,
    pub block_number: i64,
    pub payload: Vec<u8>,
    pub timestamp: chrono::NaiveDateTime,
}

/// Struct representing Notice in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "notices"]
pub struct DbNotice {
    // Numerical id of notice in database, used as cursor in connection pattern
    pub id: i32,
    pub session_id: String,
    pub epoch_index: i32,
    pub input_index: i32,
    pub notice_index: i32,
    pub proof_id: Option<i32>,
    // Keep keccak as string in database for easier db manual search
    // in ethereum hex binary format
    pub keccak: String,
    pub payload: Option<Vec<u8>>,
}

/// Struct representing Proof in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "proofs"]
pub struct DbProof {
    // Numerical id of proof in database, used as cursor in connection pattern
    pub id: i32,
    // Hashes given in Ethereum hex binary format (32 bytes), starting with '0x'
    pub output_hashes_root_hash: String,
    pub vouchers_epoch_root_hash: String,
    pub notices_epoch_root_hash: String,
    pub machine_state_hash: String,
    pub keccak_in_hashes_siblings: Vec<String>,
    pub output_hashes_in_epoch_siblings: Vec<String>,
}

/// Struct representing Voucher in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "vouchers"]
pub struct DbVoucher {
    // Numerical id of voucher in database, used as cursor in connection pattern
    pub id: i32,
    pub epoch_index: i32,
    pub input_index: i32,
    pub voucher_index: i32,
    pub proof_id: Option<i32>,
    // Keep destination as string in database for easier db manual search,
    // in ethereum hex binary format
    pub destination: String,
    pub payload: Option<Vec<u8>>,
}

/// Struct representing Report in the database
#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "reports"]
pub struct DbReport {
    // Numerical id of report in database, used as cursor in connection pattern
    pub id: i32,
    pub epoch_index: i32,
    pub input_index: i32,
    pub report_index: i32,
    // Payload is kept in the database in raw binary format
    pub payload: Option<Vec<u8>>,
}

/// Message enumeration comprising all available objects that can be kept
/// in the database
#[derive(Debug)]
pub enum Message {
    Notice(DbProof, DbNotice),
    Voucher(DbProof, DbVoucher),
    Report(DbReport),
    Input(DbInput),
}

pub fn format_endpoint(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        urlencoding::encode(postgres_user),
        urlencoding::encode(postgres_password),
        urlencoding::encode(postgres_hostname),
        postgres_port,
        urlencoding::encode(postgres_db)
    )
}

/// Create database connection manager, wait until database server is available with backoff strategy
/// Return postgres connection
pub fn connect_to_database_with_retry(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
) -> PgConnection {
    let postgres_endpoint = format_endpoint(
        postgres_hostname,
        postgres_port,
        postgres_user,
        postgres_password,
        postgres_db,
    );

    let connection_manager: ConnectionManager<PgConnection> =
        ConnectionManager::new(postgres_endpoint);

    let op = || {
        info!(
            "Trying to connect to database postgresql://{}@{}:{}/{}",
            postgres_user, postgres_hostname, postgres_port, postgres_db
        );
        connection_manager.connect().map_err(crate::new_backoff_err)
    };
    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .map_err(|e| {
            error!(
                "Failed to connect to database postgresql://{}@{}:{}/{}, error: {}",
                postgres_user, postgres_hostname, postgres_port, postgres_db,
                e.to_string()
            );
            e
        })
        .expect("Failed to connect")
}

// Connect to database in separate async blocking tread
pub async fn connect_to_database_with_retry_async(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
) -> Result<PgConnection, JoinError> {
    let postgres_hostname = postgres_hostname.to_string();
    let postgres_user = postgres_user.to_string();
    let postgres_password = postgres_password.to_string();
    let postgres_db = postgres_db.to_string();
    tokio::task::spawn_blocking(move || {
        connect_to_database_with_retry(
            postgres_hostname.as_str(),
            postgres_port,
            postgres_user.as_str(),
            postgres_password.as_str(),
            postgres_db.as_str(),
        )
    })
    .await
}

/// Create database connection pool, wait until database server is available with backoff strategy
pub fn create_db_pool_with_retry(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
) -> diesel::r2d2::Pool<ConnectionManager<PgConnection>> {
    let postgres_endpoint = format_endpoint(
        postgres_hostname,
        postgres_port,
        postgres_user,
        postgres_password,
        postgres_db,
    );

    let op = || {
        info!(
            "Trying to create db pool for database host: postgresql://{}@{}:{}/{}",
            postgres_user, postgres_hostname, postgres_port, postgres_db
        );
        diesel::r2d2::Pool::builder()
            .max_size(POOL_CONNECTION_SIZE)
            .build(ConnectionManager::<PgConnection>::new(
                postgres_endpoint.clone(),
            ))
            .map_err(crate::new_backoff_err)
    };

    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .map_err(|e| {
            error!(
                "Failed to create db pool for postgresql://{}@{}:{}/{}, error: {}",
                postgres_user, postgres_hostname, postgres_port, postgres_db,
                e.to_string()
            );
            e
        })
        .expect("error creating pool")
}

pub async fn create_db_pool_with_retry_async(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
) -> Result<diesel::r2d2::Pool<ConnectionManager<PgConnection>>, JoinError> {
    let postgres_hostname = postgres_hostname.to_string();
    let postgres_user = postgres_user.to_string();
    let postgres_password = postgres_password.to_string();
    let postgres_db = postgres_db.to_string();
    tokio::task::spawn_blocking(move || {
        create_db_pool_with_retry(
            &postgres_hostname,
            postgres_port,
            &postgres_user,
            &postgres_password,
            &postgres_db,
        )
    })
    .await
}

pub fn perform_diesel_setup(
    postgres_hostname: &str,
    postgres_port: u16,
    postgres_user: &str,
    postgres_password: &str,
    postgres_db: &str,
    migration_folder: &str,
) -> std::io::Result<std::process::Output> {
    let postgres_endpoint = format_endpoint(
        postgres_hostname,
        postgres_port,
        postgres_user,
        postgres_password,
        postgres_db,
    );

    info!(
        "Performing diesel setup on database postgresql://{}@{}:{}/{} from migration directory {}",
        postgres_user, postgres_hostname, postgres_port, postgres_db, migration_folder
    );

    std::process::Command::new("diesel")
        .arg("setup")
        .arg(&format!("--database-url={}", postgres_endpoint))
        .arg(&format!("--migration-dir={}", migration_folder))
        .output()
}
