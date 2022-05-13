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
use crate::db_service::{get_current_db_epoch_async, EpochIndexType};
use chrono::Local;
use diesel::PgConnection;
use ethers::core::types::{Address, U256};
use offchain::fold::types::{Input, PhaseState, RollupsState};
use rollups_data::database::{DbInput, DbNotice, Message};
use snafu::ResultExt;
use state_fold::types::BlockState;
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
    state_server::{
        delegate_manager_client::DelegateManagerClient, GetStateRequest,
        GetStateResponse,
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
        .context(super::error::TonicTransportError)
}

/// Connect to Cartesi State Server using backoff strategy
async fn connect_to_state_server_with_retry(
    state_server_endpoint: &str,
) -> Result<DelegateManagerClient<tonic::transport::Channel>, crate::error::Error>
{
    let endpoint = state_server_endpoint.to_string();
    let op = || async {
        DelegateManagerClient::connect(endpoint.clone())
            .await
            .map_err(rollups_data::new_backoff_err)
    };
    backoff::future::retry(backoff::ExponentialBackoff::default(), op)
        .await
        .context(super::error::TonicTransportError)
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
        .context(crate::error::TonicStatusError)?
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
        .context(crate::error::TonicStatusError)?
        .into_inner())
}

/// Get state from state server (a.k.a delegate manager)
async fn get_state(
    client: &mut DelegateManagerClient<tonic::transport::Channel>,
    initial_state: &str,
) -> Result<GetStateResponse, crate::error::Error> {
    let request = GetStateRequest {
        json_initial_state: initial_state.into(),
    };

    Ok(client
        .get_state(tonic::Request::new(request.clone()))
        .await
        .context(crate::error::TonicStatusError)?
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
                            id: 0,
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

/// Get state server current state
async fn poll_state(
    message_tx: &mpsc::Sender<Message>,
    client: &mut DelegateManagerClient<tonic::transport::Channel>,
    dapp_contract_address: Address,
    initial_epoch: i32,
) -> Result<(), crate::error::Error> {
    let initial_state = (U256::from(initial_epoch), dapp_contract_address);

    let json_initial_state = serde_json::to_string(&initial_state)
        .context(super::error::SerializeError)?;

    debug!(
        "Json state server retrieve initial state argument: {:?}",
        &json_initial_state
    );

    let state_str = get_state(client, &json_initial_state).await?;
    trace!(
        "State received for initial state {:?} is {:?}",
        &json_initial_state,
        state_str
    );

    let block_state: BlockState<RollupsState> =
        serde_json::from_str(&state_str.json_state)
            .context(super::error::DeserializeError)?;

    async fn send_input(
        index: usize,
        epoch_index: i32,
        input: &Input,
        message_tx: &mpsc::Sender<Message>,
    ) {
        if let Err(e) = message_tx
            .send(Message::Input(DbInput {
                id: 0,
                epoch_index: epoch_index,
                input_index: index as i32,
                block_number: input.block_number.as_u64() as i64,
                sender: "0x".to_string() + hex::encode(input.sender).as_str(),
                payload: (*input.payload).clone(),
                timestamp: chrono::NaiveDateTime::from_timestamp(
                    input.timestamp.low_u64() as i64,
                    0,
                ),
            }))
            .await
        {
            error!("error passing input message to db {}", e.to_string())
        }
    }

    // Process first inputs from finalized epochs
    for finalized_epoch in block_state.state.finalized_epochs.finalized_epochs {
        for (index, input) in finalized_epoch.inputs.inputs.iter().enumerate() {
            debug!("Processing finalized input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                    &input.sender, finalized_epoch.epoch_number.as_u32(), &input.block_number, &input.timestamp, &input.payload );
            send_input(
                index,
                finalized_epoch.epoch_number.as_u32() as i32,
                input,
                message_tx,
            )
            .await;
        }
    }

    if let PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch } =
        block_state.state.current_phase
    {
        for (index, input) in sealed_epoch.inputs.inputs.iter().enumerate() {
            debug!("Processing sealed epoch input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                    &input.sender, sealed_epoch.epoch_number.as_u32(), &input.block_number, &input.timestamp, &input.payload );
            send_input(
                index,
                sealed_epoch.epoch_number.as_u32() as i32,
                input,
                message_tx,
            )
            .await;
        }
    }

    for (index, input) in block_state
        .state
        .current_epoch
        .inputs
        .inputs
        .iter()
        .enumerate()
    {
        debug!("Processing current input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                    &input.sender, block_state.state.current_epoch.epoch_number.as_u32(), &input.block_number, &input.timestamp, &input.payload );
        send_input(
            index,
            block_state.state.current_epoch.epoch_number.as_u32() as i32,
            input,
            message_tx,
        )
        .await;
    }

    Ok(())
}

/// Polling task type
enum TaskType {
    GetEpochStatus,
    GetState,
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
    let postgres_endpoint = crate::db_service::format_endpoint(&config);

    let mut tasks = vec![
        Task {
            interval: poll_interval,
            next_execution: std::time::SystemTime::now().add(2 * poll_interval),
            task_type: TaskType::GetEpochStatus,
        },
        Task {
            interval: poll_interval,
            next_execution: std::time::SystemTime::now().add(poll_interval),
            task_type: TaskType::GetState,
        },
    ];

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
                    TaskType::GetState => {
                        {
                            let config = config.clone();
                            let message_tx = message_tx.clone();
                            let postgres_endpoint = postgres_endpoint.clone();
                            tokio::spawn(async move {
                                debug!(
                                    "Performing get state from state server {}",
                                    &config.state_server_endpoint
                                );

                                let mut state_server_client =
                                    match connect_to_state_server_with_retry(
                                        &config.state_server_endpoint,
                                    )
                                    .await
                                    {
                                        Ok(client) => client,
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            error!("Failed to connect to state server endpoint {}: {}", &config.state_server_endpoint, e.to_string());
                                            return (); // return from spanned background task
                                        }
                                    };
                                let db_epoch_index =
                                    get_current_db_epoch_async(
                                        &postgres_endpoint,
                                        EpochIndexType::Input,
                                    )
                                    .await
                                    .unwrap_or_default();

                                if let Err(e) = poll_state(
                                    &message_tx,
                                    &mut state_server_client,
                                    config.dapp_contract_address,
                                    db_epoch_index,
                                )
                                .await
                                {
                                    error!(
                                        "Error pooling get state from delegate server {}",
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

/// Sync data from machine manager starting from last epoch recorded
/// in database
async fn sync_epoch_status(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    mut client: ServerManagerClient<tonic::transport::Channel>,
    conn: PgConnection,
    session_id: &str,
) -> Result<(), crate::error::Error> {
    let db_epoch_index = tokio::task::spawn_blocking(move || {
        Ok(crate::db_service::get_current_db_epoch(
            &conn,
            EpochIndexType::Notice,
        )? as u64)
    })
    .await
    .map_err(|e| crate::error::Error::TokioError { source: e })??;

    let current_session_status =
        get_session_status(&mut client, session_id).await?;
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
            session_id,
            Some(epoch_index),
        )
        .await
        {
            error!("Error pooling epoch status {}", e.to_string());
        }
    }

    Ok(())
}

/// Sync data from state server starting from last epoch recorded
/// in database
async fn sync_state(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    mut client: DelegateManagerClient<tonic::transport::Channel>,
    conn: PgConnection,
    dapp_contract_address: Address,
) -> Result<(), crate::error::Error> {
    let db_epoch_index = tokio::task::spawn_blocking(move || {
        Ok(crate::db_service::get_current_db_epoch(
            &conn,
            EpochIndexType::Input,
        )? as u64)
    })
    .await
    .context(crate::error::TokioError)??;

    // Pool epoch state for all previous epochs
    debug!(
        "Syncing in progress, polling state since epoch {}",
        db_epoch_index
    );
    if let Err(e) = poll_state(
        &message_tx,
        &mut client,
        dapp_contract_address,
        db_epoch_index as i32,
    )
    .await
    {
        error!("Error pooling epoch status {}", e.to_string());
    }

    Ok(())
}

async fn sync_data(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    config: Arc<IndexerConfig>,
) -> Result<(), crate::error::Error> {
    let postgres_endpoint = crate::db_service::format_endpoint(&config);

    // Sync notices from machine manager
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        postgres_endpoint.clone(),
    )
    .await
    .context(crate::error::TokioError)?;

    let client =
        connect_to_server_manager_with_retry(&config.mm_endpoint).await?;
    sync_epoch_status(message_tx.clone(), client, conn, &config.session_id)
        .await?;

    // Sync inputs and epochs from state server
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        postgres_endpoint.clone(),
    )
    .await
    .context(crate::error::TokioError)?;

    let client =
        connect_to_state_server_with_retry(&config.state_server_endpoint)
            .await?;
    sync_state(
        message_tx.clone(),
        client,
        conn,
        config.dapp_contract_address,
    )
    .await?;

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
                "Data successfully synced from Cartesi server manager {} and state server {}",
                &config.mm_endpoint, &config.state_server_endpoint
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
