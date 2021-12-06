use crate::error::*;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use snafu::ResultExt;

pub type PollingPool = Pool<ConnectionManager<PgConnection>>;
pub type Connection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn val_to_hex_str<T: std::fmt::LowerHex>(val: &T) -> String {
    format!("{:#x}", val)
}

pub fn create_pool(postgres_endpoint: String) -> Result<PollingPool> {
    let connection_manager = ConnectionManager::new(&postgres_endpoint);

    Ok(Pool::new(connection_manager).context(R2D2Error)?)
}
