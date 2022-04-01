mod config;
mod error;
pub mod graphql;
pub mod http;

use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let reader_config = config::ReaderConfig::initialize().map_err(|e| {
        crate::error::Error::BadConfiguration {
            err: format!("Fail to initialize reader config: {}", e.to_string()),
        }
    })?;

    info!(
        "Starting graphql reader service on {}:{}",
        reader_config.graphql_host, reader_config.graphql_port
    );

    let database_url = "postgres://".to_string()
        + reader_config.db_username.as_str()
        + ":"
        + reader_config.db_password.as_str()
        + "@"
        + reader_config.db_host.as_str()
        + ":"
        + reader_config.db_port.to_string().as_str()
        + "/"
        + reader_config.db_name.as_str();
    info!(
        "Postgres database host: {}:{}/{}",
        &reader_config.db_host, &reader_config.db_port, &reader_config.db_name
    );

    let db_pool =
        rollups_data::database::create_db_pool_with_retry(&database_url).await;

    // Start http server
    match tokio::try_join!(http::start_service(
        &reader_config.graphql_host,
        reader_config.graphql_port,
        db_pool
    )) {
        Ok(_) => {
            info!("reader service terminated successfully");
            Ok(())
        }
        Err(e) => {
            warn!("reader service terminated with error: {}", e);
            Err(crate::error::Error::HttpServiceError { source: e })
        }
    }
}
