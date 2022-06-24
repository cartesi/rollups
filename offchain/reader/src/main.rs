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

mod config;

use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> Result<(), reader::error::Error> {
    // Use tracing library for logs. By default use system standard output logger

    // Configure a custom event formatter
    let tracing_format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(true)
        .with_target(true)
        .with_ansi(false)
        .compact();

    if std::env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_max_level(LevelFilter::INFO)
            .init();
    }

    let reader_config = config::ReaderConfig::initialize().map_err(|e| {
        reader::error::Error::BadConfiguration {
            err: format!("Fail to initialize reader config: {}", e),
        }
    })?;

    info!(
        "Starting graphql reader service on {}:{}",
        reader_config.graphql_host, reader_config.graphql_port
    );

    info!(
        "Using postgres database host: {}:{}/{}",
        &reader_config.postgres_hostname,
        &reader_config.postgres_port,
        &reader_config.postgres_db
    );

    let db_pool = rollups_data::database::create_db_pool_with_retry(
        &reader_config.postgres_hostname,
        reader_config.postgres_port,
        &reader_config.postgres_user,
        &reader_config.postgres_password,
        &reader_config.postgres_db,
    );

    // Start http server
    match tokio::try_join!(reader::http::start_service(
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
            Err(reader::error::Error::HttpServiceError { source: e })
        }
    }
}
