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

/// Data service polls other Cartesi services
/// with specified interval and collects related data about dapp
use crate::config::IndexerConfig;
use chrono::Local;
use rollups_data::database::{DbNotice, Message};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

use crate::grpc::{
    cartesi_machine, cartesi_server_manager,
    cartesi_server_manager::{
        server_manager_client::ServerManagerClient, GetEpochStatusRequest,
        GetEpochStatusResponse, GetSessionStatusRequest,
        GetSessionStatusResponse,
    },
};

/// Connect to Cartesi server manager using backoff strategy
async fn connect_to_server_manager_with_retry(
    mm_endpoint: &str,
) -> Result<ServerManagerClient<tonic::transport::Channel>, crate::error::Error>
{
    let endpoint = mm_endpoint.to_string();
    let op = || async {
        ServerManagerClient::connect(endpoint.clone())
            .await
            .map_err(rollups_data::new_backoff_err)
    };
    backoff::future::retry(backoff::ExponentialBackoff::default(), op)
        .await
        .map_err(|e| crate::error::Error::TonicTransportError { source: e })
}

/// Get epoch status from server manager
async fn get_epoch_status(
    client: &mut ServerManagerClient<tonic::transport::Channel>,
    session_id: &str,
    epoch_index: u64,
) -> Result<GetEpochStatusResponse, crate::error::Error> {
    let request = GetEpochStatusRequest {
        session_id: session_id.to_string(),
        epoch_index,
    };

    Ok(client
        .get_epoch_status(tonic::Request::new(request.clone()))
        .await
        .map_err(|e| crate::error::Error::TonicStatusError { source: e })?
        .into_inner())
}

/// Get session status from server manager
async fn get_session_status(
    client: &mut ServerManagerClient<tonic::transport::Channel>,
    session_id: &str,
) -> Result<GetSessionStatusResponse, crate::error::Error> {
    let request = GetSessionStatusRequest {
        session_id: session_id.to_string(),
    };

    Ok(client
        .get_session_status(tonic::Request::new(request.clone()))
        .await
        .map_err(|e| crate::error::Error::TonicStatusError { source: e })?
        .into_inner())
}

/// Get epoch status and send relevant data to db service
/// If epoch_index is not provided, use session active epoch index
async fn poll_epoch_status(
    message_tx: &mpsc::Sender<Message>,
    client: &mut ServerManagerClient<tonic::transport::Channel>,
    session_id: &str,
    epoch_index: Option<u64>,
) -> Result<(), crate::error::Error> {
    let epoch_index: u64 = match epoch_index {
        Some(index) => index,
        None => {
            let session_status = get_session_status(client, session_id).await?;
            debug!(
                "Retrieving current epoch index, acquired session status {:?}",
                session_status
            );
            session_status.active_epoch_index
        }
    };

    let epoch_status_response =
        get_epoch_status(client, session_id, epoch_index).await?;

    for input in epoch_status_response.processed_inputs {
        debug!(
            "Processed epoch {} input {} reports len: {}",
            epoch_index,
            input.input_index,
            input.reports.len()
        );
        if let Some(one_of) = input.processed_oneof {
            match one_of {
                cartesi_server_manager::processed_input::ProcessedOneof::Result(input_result) => {
                    // Process notices
                    for (nindex, notice) in input_result.notices.iter().enumerate() {
                        // Send one notice to database service
                        trace!("Sending notice with session id {}, epoch_index {} input_index {} notice_index {}",
                                session_id, epoch_index, input.input_index, &nindex);
                        if let Err(e) = message_tx.send(Message::Notice(DbNotice {
                            session_id: session_id.to_string(),
                            epoch_index: epoch_index as i32,
                            input_index: input.input_index  as i32,
                            notice_index: nindex as i32,
                            keccak: hex::encode(
                                &notice
                                    .keccak
                                    .as_ref()
                                    .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                    .data,
                            ),
                            payload: Some(notice.payload.clone()),
                            timestamp: Local::now()
                        })).await {
                            error!("error passing message to db {}", e.to_string())
                        }
                    }
                },
                cartesi_server_manager::processed_input::ProcessedOneof::SkipReason(reason) => {
                    info!("Skip input for reason {}", reason);
                }
            }
        }
    }
    Ok(())
}

/// Polling task type
enum TaskType {
    GetEpochStatus,
}

/// Task that gets executed in the polling event loop
struct Task {
    interval: std::time::Duration,
    next_execution: std::time::SystemTime,
    task_type: TaskType,
}

async fn polling_loop(
    config: Arc<IndexerConfig>,
    message_tx: mpsc::Sender<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    let loop_interval = std::time::Duration::from_millis(100);
    let poll_interval = std::time::Duration::from_secs(config.interval);

    let mut tasks = vec![Task {
        interval: poll_interval,
        next_execution: std::time::SystemTime::now().add(poll_interval),
        task_type: TaskType::GetEpochStatus,
    }];

    // Polling tasks loop
    info!("Starting data polling loop");
    loop {
        for task in tasks.iter_mut() {
            if task.next_execution <= std::time::SystemTime::now() {
                match task.task_type {
                    TaskType::GetEpochStatus => {
                        {
                            let config = config.clone();
                            let message_tx = message_tx.clone();
                            tokio::spawn(async move {
                                debug!("Performing get epoch status from Cartesi server manager {}", &config.mm_endpoint);
                                let mut server_manager_client =
                                    match connect_to_server_manager_with_retry(
                                        &config.mm_endpoint,
                                    )
                                    .await
                                    {
                                        Ok(client) => client,
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            error!("Failed to connect to server manager endpoint {}: {}", &config.mm_endpoint, e.to_string());
                                            return (); // return from spanned background task
                                        }
                                    };
                                if let Err(e) = poll_epoch_status(
                                    &message_tx,
                                    &mut server_manager_client,
                                    &config.session_id,
                                    None,
                                )
                                .await
                                {
                                    error!(
                                        "Error pooling epoch status {}",
                                        e.to_string()
                                    );
                                }
                            });
                        }
                        task.next_execution =
                            task.next_execution.add(task.interval);
                    }
                }
            }
        }
        tokio::time::sleep(loop_interval).await;
    }
}

/// Sync data from external endpoints on indexer startup,
/// based on current database epoch index
async fn sync_data(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    config: Arc<IndexerConfig>,
) -> Result<(), crate::error::Error> {
    let postgres_endpoint = crate::db_service::format_endpoint(&config);
    let conn = tokio::task::spawn_blocking(move || {
        rollups_data::database::connect_to_database_with_retry(
            &postgres_endpoint,
        )
    })
    .await
    .map_err(|e| crate::error::Error::TokioError { source: e })?;

    let db_epoch_index = tokio::task::spawn_blocking(move || {
        Ok(crate::db_service::get_current_db_epoch(&conn)? as u64)
    })
    .await
    .map_err(|e| crate::error::Error::TokioError { source: e })??;

    let mut client =
        connect_to_server_manager_with_retry(&config.mm_endpoint).await?;
    let current_session_status =
        get_session_status(&mut client, &config.session_id).await?;
    debug!(
        "Sync initial epochs: database epoch index is {}, session active epoch index is {}",
        db_epoch_index, current_session_status.active_epoch_index
    );

    // Pool epoch status for all previous epochs
    for epoch_index in
        db_epoch_index..=current_session_status.active_epoch_index
    {
        debug!(
            "Syncing in progress, polling epoch status for epoch {}",
            epoch_index
        );
        if let Err(e) = poll_epoch_status(
            &message_tx,
            &mut client,
            &config.session_id,
            Some(epoch_index),
        )
        .await
        {
            error!("Error pooling epoch status {}", e.to_string());
        }
    }

    Ok(())
}

/// Create and run new instance of data polling service
pub async fn run(
    config: IndexerConfig,
    message_tx: mpsc::Sender<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    let config = Arc::new(config);
    match sync_data(message_tx.clone(), config.clone()).await {
        Ok(()) => {
            info!(
                "Data successfully synced from Cartesi server manager {}",
                &config.mm_endpoint
            );
        }
        Err(e) => {
            error!(
                "Failed to sync data from Cartesi server manager {}: {}",
                &config.mm_endpoint,
                e.to_string()
            );
        }
    }
    // Start polling loop
    polling_loop(config, message_tx).await
}
