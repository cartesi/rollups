#![allow(dead_code)]

pub mod schema;

use chrono::{DateTime, Local, Utc};
use diesel::backend::Backend;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, ManageConnection};
use diesel::{Insertable, Queryable};
use schema::notices;

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "notices"]
pub struct DbNotice {
    pub session_id: String,
    pub epoch_index: i32,
    pub input_index: i32,
    pub notice_index: i32,
    pub keccak: String,
    pub payload: Option<Vec<u8>>,
    #[diesel(deserialize_as = "LocalDateTimeWrapper")]
    pub timestamp: chrono::DateTime<chrono::Local>,
}
pub struct LocalDateTimeWrapper(DateTime<Local>);
impl Into<DateTime<Local>> for LocalDateTimeWrapper {
    fn into(self) -> DateTime<Local> {
        self.0
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

#[derive(Debug)]
pub enum Message {
    Notice(DbNotice),
}

pub const POOL_CONNECTION_SIZE: u32 = 3;

fn new_backoff_err<E: std::fmt::Display>(err: E) -> backoff::Error<E> {
    // Retry according to backoff policy
    backoff::Error::Transient {
        err,
        retry_after: None,
    }
}

/// Create database connection manager, wait until database server is available with backoff strategy
pub async fn connect_to_database_with_retry(
    connection_manager: &ConnectionManager<PgConnection>,
) -> PgConnection {
    let op = || connection_manager.connect().map_err(new_backoff_err);
    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .expect("Failed to connect")
}

/// Create pool, wait until database server is available with backoff strategy
pub fn create_db_pool_with_retry(
    database_url: &str,
) -> diesel::r2d2::Pool<ConnectionManager<PgConnection>> {
    let op = || {
        diesel::r2d2::Pool::builder()
            .max_size(POOL_CONNECTION_SIZE)
            .build(ConnectionManager::<PgConnection>::new(database_url))
            .map_err(new_backoff_err)
    };

    backoff::retry(backoff::ExponentialBackoff::default(), op)
        .expect("error creating pool")
}
