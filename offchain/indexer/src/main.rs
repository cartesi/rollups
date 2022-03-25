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

use indexer::config::IndexerConfig;
use indexer::data_service;
use indexer::db_service;

use indexer::error::Error::BadConfiguration;
use tokio::sync::mpsc;
use tracing::{error, info, trace};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting rollup indexer service");
    let indexer_config =
        IndexerConfig::initialize().map_err(|e| BadConfiguration {
            err: format!("Fail to initialize indexer config: {}", e),
        })?;

    trace!("Indexer configuration {:?}", &indexer_config);

    let (message_tx, message_rx) = mpsc::channel::<db_service::Message>(128);
    tokio::select! {
        db_service_result = db_service::run(indexer_config.clone(), message_rx) => {
            match db_service_result {
                Ok(_) => info!("db service terminated successfully"),
                Err(e) => error!("db service terminated with error: {}", e)
            }
        },
        data_service_result = data_service::run(indexer_config.clone(), message_tx) => {
            match data_service_result {
                Ok(_) => info!("data service terminated successfully"),
                Err(e) => error!("data service terminated with error: {}", e)
            }
        }
    }
    info!("Ended rollups indexer service");
    Ok(())
}
