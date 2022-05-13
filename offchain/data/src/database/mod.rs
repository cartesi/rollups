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

use chrono::{DateTime, Local, Utc};
use diesel::backend::Backend;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, ManageConnection};
use diesel::{Insertable, Queryable};
use schema::{epochs, inputs, notices};
use tokio::task::JoinError;

pub const CURRENT_NOTICE_EPOCH_INDEX: &str = "current_notice_epoch_index";
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
    // Keep keccak as string in database for easier db manual search
    pub keccak: String,
    pub payload: Option<Vec<u8>>,
    #[diesel(deserialize_as = "LocalDateTimeWrapper")]
    pub timestamp: chrono::DateTime<chrono::Local>, //todo use time from input
}

pub struct LocalDateTimeWrapper(DateTime<Local>);
impl From<LocalDateTimeWrapper> for DateTime<Local> {
    fn from(wrapper: LocalDateTimeWrapper) -> DateTime<Local> {
        wrapper.0
    }
}

impl<DB, ST> Queryable<ST, DB> for LocalDateTimeWrapper
where
    DB: Backend,
    DateTime<Utc>: Queryable<ST, DB>,
{
    type Row = <DateTime<Utc> as Queryable<ST, DB>>::Row;

    fn build(row: Self::Row) -> Self {
        Self(
            <DateTime<Utc> as Queryable<ST, DB>>::build(row)
                .with_timezone(&Local),
        )
    }
}

/// Message enumeration comprising all available objects that can be kept
/// in the database
#[derive(Debug)]
pub enum Message {
    Notice(DbNotice),
    Input(DbInput),
}

/// Create database connection manager, wait until database server is available with backoff strategy
/// Return postgres connection
pub fn connect_to_database_with_retry(postgres_endpoint: &str) -> PgConnection {
    let connection_manager: ConnectionManager<PgConnection> =
        ConnectionManager::new(postgres_endpoint);

    let op = || connection_manager.connect().map_err(crate::new_backoff_err);
    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .expect("Failed to connect")
}

/// Create database connection pool, wait until database server is available with backoff strategy
pub fn create_db_pool_with_retry(
    database_url: &str,
) -> diesel::r2d2::Pool<ConnectionManager<PgConnection>> {
    let op = || {
        diesel::r2d2::Pool::builder()
            .max_size(POOL_CONNECTION_SIZE)
            .build(ConnectionManager::<PgConnection>::new(database_url))
            .map_err(crate::new_backoff_err)
    };

    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .expect("error creating pool")
}

// Connect to database in separate async blocking tread
pub async fn connect_to_database_with_retry_async(
    endpoint: String,
) -> Result<PgConnection, JoinError> {
    tokio::task::spawn_blocking(move || {
        connect_to_database_with_retry(&endpoint)
    })
    .await
}
