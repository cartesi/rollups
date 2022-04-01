#![allow(dead_code)]

pub mod schema;

use chrono::{DateTime, Local, Utc};
use diesel::backend::Backend;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, ManageConnection};
use diesel::{Insertable, Queryable};
use schema::notices;
use tracing::error;

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

pub const MIN_CONNECTION_RETRY_PERIOD: u64 = 1; //seconds
pub const MAX_CONNECTION_RETRY_PERIOD: u64 = 60; //seconds
pub const POOL_CONNECTION_SIZE: u32 = 3;

fn connect(
    connection_manager: &ConnectionManager<PgConnection>,
    connection_retry: &mut u64,
) -> Result<PgConnection, diesel::r2d2::Error> {
    match connection_manager.connect() {
        Ok(conn) => {
            *connection_retry = MIN_CONNECTION_RETRY_PERIOD;
            Ok(conn)
        }
        Err(e) => {
            if *connection_retry < MAX_CONNECTION_RETRY_PERIOD / 2 {
                *connection_retry = *connection_retry * 2;
            } else {
                *connection_retry = MAX_CONNECTION_RETRY_PERIOD;
            }
            Err(e)
        }
    }
}

/// Create database connection manager, wait until database server is available with backoff strategy
pub async fn connect_to_database_with_retry(
    connection_manager: &ConnectionManager<PgConnection>,
) -> PgConnection {
    let mut connection_retry_period = MIN_CONNECTION_RETRY_PERIOD; //seconds
    loop {
        match connect(connection_manager, &mut connection_retry_period) {
            Ok(connection) => break connection,
            Err(e) => {
                error!(
                    "Unable to connect to database, error {}",
                    e.to_string()
                );
                tokio::time::sleep(std::time::Duration::from_secs(
                    connection_retry_period,
                ))
                .await;
                continue;
            }
        };
    }
}

/// Create pool, wait until database server is available with backoff strategy
pub async fn create_db_pool_with_retry(
    database_url: &str,
) -> diesel::r2d2::Pool<ConnectionManager<PgConnection>> {
    let mut connection_retry_period = MIN_CONNECTION_RETRY_PERIOD; //seconds
    loop {
        match diesel::r2d2::Pool::builder()
            .max_size(POOL_CONNECTION_SIZE)
            .build(ConnectionManager::<PgConnection>::new(database_url))
        {
            Ok(pool) => break pool,
            Err(e) => {
                error!(
                    "Unable to connect to database, error {}",
                    e.to_string()
                );
                tokio::time::sleep(std::time::Duration::from_secs(
                    connection_retry_period,
                ))
                .await;
                if connection_retry_period < MAX_CONNECTION_RETRY_PERIOD / 2 {
                    connection_retry_period = connection_retry_period * 2;
                } else {
                    connection_retry_period = MAX_CONNECTION_RETRY_PERIOD;
                }
                continue;
            }
        };
    }
}
