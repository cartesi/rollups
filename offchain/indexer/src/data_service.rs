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
use crate::config::{IndexerConfig, PostgresConfig};
use crate::db_service::EpochIndexType;
use crate::error::new_indexer_tokio_err;
use crate::http::HealthStatus;
use async_mutex::Mutex;
use rollups_data::database::{
    DbInput, DbNotice, DbProof, DbReport, DbVoucher, Message,
};
use snafu::ResultExt;
use state_fold_types::{ethabi::ethereum_types::Address, QueryBlock};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};
use types::foldables::{
    authority::rollups::{RollupsInitialState, RollupsState},
    input_box::Input,
};
use uuid::Uuid;

use state_client_lib::{GrpcStateFoldClient, StateServer};

use crate::grpc::{
    cartesi_machine, server_manager,
    server_manager::{
        server_manager_client::ServerManagerClient, EpochState,
        GetEpochStatusRequest, GetEpochStatusResponse, GetSessionStatusRequest,
        GetSessionStatusResponse,
    },
};

type RollupsStateServer =
    GrpcStateFoldClient<RollupsInitialState, RollupsState>;

/// Connect to Cartesi server manager using backoff strategy
async fn connect_to_server_manager_with_retry(
    mm_endpoint: &str,
) -> Result<ServerManagerClient<tonic::transport::Channel>, crate::error::Error>
{
    let endpoint = mm_endpoint.to_string();
    debug!("Connecting to server manager {}", endpoint);
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
) -> Result<RollupsStateServer, crate::error::Error> {
    let endpoint = state_server_endpoint.to_string();
    debug!("Connecting to state server {}", endpoint);
    let op = || async {
        tonic::transport::Channel::from_shared(state_server_endpoint.to_owned())
            .expect(&format!(
                "invalid state-fold-server uri {}",
                state_server_endpoint
            ))
            .connect()
            .await
            .map(GrpcStateFoldClient::new_from_channel)
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
    let request_id = Uuid::new_v4().to_string();
    let request = GetEpochStatusRequest {
        session_id: session_id.to_string(),
        epoch_index,
    };

    tracing::trace!(request_id, ?request, "calling grpc get_epoch_status");

    let mut grpc_request = tonic::Request::new(request);
    grpc_request
        .metadata_mut()
        .insert("request-id", request_id.parse().unwrap());
    let response = client.get_epoch_status(grpc_request).await;

    tracing::trace!(
        request_id,
        ?response,
        "got grpc response from get_epoch_status"
    );

    response
        .map(|v| v.into_inner())
        .context(crate::error::TonicStatusError)
}

/// Get session status from server manager
async fn get_session_status(
    client: &mut ServerManagerClient<tonic::transport::Channel>,
    session_id: &str,
) -> Result<GetSessionStatusResponse, crate::error::Error> {
    let request_id = Uuid::new_v4().to_string();
    let request = GetSessionStatusRequest {
        session_id: session_id.to_string(),
    };

    tracing::trace!(request_id, ?request, "calling grpc get_session_status");

    let mut grpc_request = tonic::Request::new(request);
    grpc_request
        .metadata_mut()
        .insert("request-id", request_id.parse().unwrap());
    let response = client.get_session_status(grpc_request).await;

    tracing::trace!(
        request_id,
        ?response,
        "got grpc response from get_session_status"
    );

    response
        .map(|v| v.into_inner())
        .context(crate::error::TonicStatusError)
}

async fn process_epoch_status_response(
    epoch_status_response: GetEpochStatusResponse,
    message_tx: &mpsc::Sender<Message>,
    session_id: &str,
    epoch_index: u64,
) -> Result<u64, crate::error::Error> {
    let epoch_state = epoch_status_response.state();
    for input in epoch_status_response.processed_inputs {
        // Process reports
        for (rindex, report) in input.reports.iter().enumerate() {
            trace!("Process epoch status: sending report with session id {}, epoch_index {} input_index {} report_index {} report {:?}",
                                session_id, epoch_index, input.input_index, &rindex, report);
            if let Err(e) = message_tx
                .send(Message::Report(DbReport {
                    id: 0,
                    epoch_index: epoch_index as i32,
                    input_index: input.input_index as i32,
                    report_index: rindex as i32,
                    // Keep payload in database in raw byte format
                    payload: Some(report.payload.clone()),
                }))
                .await
            {
                error!(
                    "Process epoch status: error passing report message to db {}",
                    e.to_string()
                )
            }
        }

        if let Some(one_of) = input.processed_input_one_of {
            match one_of {
                server_manager::processed_input::ProcessedInputOneOf::AcceptedData(accepted_data) => {
                    // Process vouchers
                    for (vindex, voucher) in accepted_data.vouchers.iter().enumerate() {
                        // Send one voucher to database service
                        trace!("Process epoch status: sending voucher with session id {}, epoch_index {} input_index {} voucher_index {} voucher {:?}",
                                session_id, epoch_index, input.input_index, &vindex, voucher);

                        // Construct and send voucher proof only when epoch is finished
                        let proof = match epoch_state {
                            EpochState::Finished => {
                                Some(DbProof {
                                    id: 0,
                                    machine_state_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_machine_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    vouchers_epoch_root_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_vouchers_epoch_root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    notices_epoch_root_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_notices_epoch_root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    output_hashes_root_hash:  "0x".to_string() + hex::encode(&voucher.keccak_in_voucher_hashes
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    keccak_in_hashes_siblings: voucher.keccak_in_voucher_hashes
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .sibling_hashes
                                        .iter()
                                        .rev()
                                        .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect(),
                                    output_hashes_in_epoch_siblings:    input.voucher_hashes_in_epoch
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .sibling_hashes
                                        .iter()
                                        .rev()
                                        .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect()
                                })
                            },
                            EpochState::Active => None,
                        };

                        // Send voucher to db service
                        if let Err(e) = message_tx.send(Message::Voucher(proof, DbVoucher {
                            id: 0,
                            epoch_index: epoch_index as i32,
                            input_index: input.input_index  as i32,
                            voucher_index: vindex as i32,
                            proof_id: None,
                            // Encode destination in hex format, to be able to easily query it in the database
                            destination:  "0x".to_string() + hex::encode(
                                &voucher
                                    .address
                                    .as_ref()
                                    .unwrap_or(&server_manager::Address { data: vec![] })
                                    .data,
                            ).as_str(),
                            // Payload is in raw format
                            payload: Some(voucher.payload.clone())
                        })).await {
                            error!("Process epoch status: error passing voucher message to db {}", e.to_string())
                        }
                    }
                    // Process notices
                    for (nindex, notice) in accepted_data.notices.iter().enumerate() {
                        // Send one notice to database service
                        trace!("Process epoch status: sending notice with session id {}, epoch_index {} input_index {} notice_index {}",
                                session_id, epoch_index, input.input_index, &nindex);

                        // Construct and send notice proof only when epoch is finished
                        let proof = match epoch_state {
                            EpochState::Finished => {
                                Some(DbProof {
                                    id: 0,
                                    machine_state_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_machine_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    vouchers_epoch_root_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_vouchers_epoch_root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    notices_epoch_root_hash: "0x".to_string() + hex::encode(&epoch_status_response
                                        .most_recent_notices_epoch_root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    output_hashes_root_hash:  "0x".to_string() + hex::encode(&notice.keccak_in_notice_hashes
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .root_hash
                                        .as_ref()
                                        .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                        .data,
                                    ).as_str(),
                                    keccak_in_hashes_siblings: notice.keccak_in_notice_hashes
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .sibling_hashes
                                        .iter()
                                        .rev()
                                        .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect(),
                                    output_hashes_in_epoch_siblings:    input.notice_hashes_in_epoch
                                        .as_ref()
                                        .unwrap_or(&Default::default())
                                        .sibling_hashes
                                        .iter()
                                        .rev()
                                        .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect()
                                })
                            },
                            EpochState::Active => None,
                        };

                        // Send notice to db service
                        if let Err(e) = message_tx.send(Message::Notice(proof, DbNotice {
                            id: 0,
                            session_id: session_id.to_string(),
                            epoch_index: epoch_index as i32,
                            input_index: input.input_index  as i32,
                            notice_index: nindex as i32,
                            // Encode keccak in hex format, to be able to easily query it in the database
                            proof_id: None,
                            keccak:  "0x".to_string() + hex::encode(
                                &notice
                                    .keccak
                                    .as_ref()
                                    .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                                    .data,
                            ).as_str(),
                            // Payload is in raw format
                            payload: Some(notice.payload.clone())
                        })).await {
                            error!("Process epoch status: error passing notice message to db {}", e.to_string())
                        }
                    }
                }
                server_manager::processed_input::ProcessedInputOneOf::ExceptionData(data) => {
                    error!("Process epoch status: exception data returned {:?}", data);
                }
            }
        }
    }
    Ok(epoch_index)
}

/// Get epoch status and send relevant data to db service
/// If epoch_index is not provided, use session active epoch index
/// Return epoch index of the epoch pooled
async fn poll_epoch_status(
    message_tx: &mpsc::Sender<Message>,
    client: &mut ServerManagerClient<tonic::transport::Channel>,
    session_id: &str,
    epoch_index: Option<u64>,
) -> Result<u64, crate::error::Error> {
    let epoch_index: u64 = match epoch_index {
        Some(index) => index,
        None => {
            let session_status = get_session_status(client, session_id).await?;
            debug!(
                "Poll epoch status: retrieving current epoch index, acquired session status {:?}",
                session_status
            );
            session_status.active_epoch_index
        }
    };

    debug!("Poll epoch status: polling for epoch {}", epoch_index);
    let epoch_status_response =
        get_epoch_status(client, session_id, epoch_index).await?;

    process_epoch_status_response(
        epoch_status_response,
        message_tx,
        session_id,
        epoch_index,
    )
    .await
}

async fn process_state_response(
    rollups_state: Arc<RollupsState>,
    dapp_address: &Address,
    message_tx: &mpsc::Sender<Message>,
) -> Result<(), crate::error::Error> {
    async fn send_input(
        index: usize,
        input: &Input,
        message_tx: &mpsc::Sender<Message>,
    ) {
        if let Err(e) = message_tx
            .send(Message::Input(DbInput {
                id: 0,
                epoch_index: 0,
                input_index: index as i32,
                block_number: input.block_added.number.as_u64() as i64,
                sender: "0x".to_string()
                    + hex::encode(input.sender.as_ref()).as_str(),
                tx_hash: None,
                payload: input.payload.clone(),
                timestamp: chrono::NaiveDateTime::from_timestamp_opt(
                    input.block_added.timestamp.low_u64() as i64,
                    0,
                )
                .expect("expect valid timestamp in input's block_added"),
            }))
            .await
        {
            error!(
                "Poll state: error passing input message to db {}",
                e.to_string()
            )
        }
    }

    // Process inputs
    if let Some(dapp_input_box) =
        rollups_state.input_box.dapp_input_boxes.get(dapp_address)
    {
        for (index, input) in dapp_input_box.inputs.iter().enumerate() {
            debug!(
                "Poll state: processing input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                &input.sender,
                0,
                &input.block_added.number,
                &input.block_added.timestamp,
                &input.payload
            );
            send_input(index, input, message_tx).await;
        }
    }

    Ok(())
}

/// Get state server current state
/// Process inputs ad send to db service
async fn poll_state(
    message_tx: &mpsc::Sender<Message>,
    client: &RollupsStateServer,
    dapp_address: &Address,
    history_address: &Address,
    input_box_address: &Address,
    depth: usize,
) -> Result<(), crate::error::Error> {
    let initial_state = RollupsInitialState {
        history_address: *history_address,
        input_box_address: *input_box_address,
    };

    debug!(
        "Poll state: state server retrieve initial state argument: {:?}",
        &initial_state
    );

    let state = client
        .query_state(&initial_state, QueryBlock::BlockDepth(depth))
        .await
        .context(super::error::StateFoldServerError)?;

    trace!(
        "Poll state: state received for initial state {:?} is {:?}",
        &initial_state,
        state
    );

    process_state_response(state.state, dapp_address, message_tx).await
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
    health_status: Arc<async_mutex::Mutex<HealthStatus>>,
) -> Result<(), crate::error::Error> {
    let loop_interval = std::time::Duration::from_millis(100);
    let poll_interval = std::time::Duration::from_secs(config.interval);

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

    // Create database db pool
    let db_config = &config.database;
    health_status.lock().await.postgres = Err(format!(
        "Trying to create db pool for database host: postgresql://{}@{}:{}/{}",
        &db_config.postgres_user,
        &db_config.postgres_hostname,
        db_config.postgres_port,
        &db_config.postgres_db
    ));

    health_status.lock().await.postgres = Ok(());

    // Polling tasks loop
    info!("Starting data polling loop");
    let last_epoch_status_index: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    loop {
        for task in tasks.iter_mut() {
            let health_status = health_status.clone();
            if task.next_execution <= std::time::SystemTime::now() {
                match task.task_type {
                    TaskType::GetEpochStatus => {
                        {
                            let config = config.clone();
                            let message_tx = message_tx.clone();
                            let last_epoch_index =
                                last_epoch_status_index.clone();
                            tokio::spawn(async move {
                                debug!("Performing get epoch status from Cartesi server manager {}", &config.mm_endpoint);
                                let mut server_manager_client =
                                    match connect_to_server_manager_with_retry(
                                        &config.mm_endpoint,
                                    )
                                    .await
                                    {
                                        Ok(client) => {
                                            health_status
                                                .lock()
                                                .await
                                                .server_manager = Ok(());
                                            client
                                        }
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            let err_message = format!("Failed to connect to server manager endpoint {}: {}", &config.mm_endpoint, e);
                                            error!("{}", &err_message);
                                            health_status
                                                .lock()
                                                .await
                                                .server_manager =
                                                Err(err_message.clone());
                                            return; // return from spanned background task
                                        }
                                    };
                                match poll_epoch_status(
                                    &message_tx,
                                    &mut server_manager_client,
                                    &config.session_id,
                                    None,
                                )
                                .await
                                {
                                    Ok(new_epoch_index) => {
                                        let mut last_epoch_index =
                                            last_epoch_index.lock().await;
                                        if new_epoch_index > *last_epoch_index {
                                            // Epoch changed, pool previous epoch just in case not to miss anything
                                            for epoch_index in *last_epoch_index
                                                ..new_epoch_index
                                            {
                                                if let Err(e) = poll_epoch_status(
                                                    &message_tx,
                                                    &mut server_manager_client,
                                                    &config.session_id,
                                                    Some(epoch_index),
                                                )
                                                    .await
                                                {
                                                    let err_message = format!("Error polling epoch status for epoch {}: {}", epoch_index, e);
                                                    error!("{}", err_message);
                                                    health_status
                                                        .lock()
                                                        .await
                                                        .server_manager =
                                                        Err(err_message);
                                                } else {
                                                    health_status
                                                        .lock()
                                                        .await
                                                        .server_manager = Ok(());
                                                }
                                            }
                                            *last_epoch_index = new_epoch_index;
                                        }
                                    }
                                    Err(e) => {
                                        let last_epoch_index =
                                            *last_epoch_index.lock().await;
                                        let err_message = format!("Error polling epoch status for epoch index {}: {}", last_epoch_index, e);
                                        error!("{}", &err_message);
                                        health_status
                                            .lock()
                                            .await
                                            .server_manager = Err(err_message);
                                    }
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
                                        Ok(client) => {
                                            health_status
                                                .lock()
                                                .await
                                                .state_server = Ok(());
                                            client
                                        }
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            let err_message = format!("Failed to connect to state server endpoint {}: {}", &config.state_server_endpoint, e);
                                            error!("{}", &err_message);
                                            health_status
                                                .lock()
                                                .await
                                                .state_server =
                                                Err(err_message);
                                            return; // return from spanned background task
                                        }
                                    };

                                if let Err(e) = poll_state(
                                    &message_tx,
                                    &mut state_server_client,
                                    &config.dapp_deployment.dapp_address,
                                    &config.rollups_deployment.history_address,
                                    &config
                                        .rollups_deployment
                                        .input_box_address,
                                    config.confirmations,
                                )
                                .await
                                {
                                    let err_message = format!("Error polling get state from state fold server {}", e);
                                    error!("{}", &err_message);
                                    health_status.lock().await.state_server =
                                        Err(err_message);
                                } else {
                                    health_status.lock().await.state_server =
                                        Ok(());
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
    postgres_config: &PostgresConfig,
    session_id: &str,
) -> Result<(), crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        &postgres_config.postgres_hostname,
        postgres_config.postgres_port,
        &postgres_config.postgres_user,
        &postgres_config.postgres_password,
        &postgres_config.postgres_db,
    )
    .await
    .context(crate::error::TokioError)?;

    let db_epoch_index = tokio::task::spawn_blocking(move || {
        Ok(crate::db_service::get_current_db_epoch(
            &conn,
            EpochIndexType::Notice,
        )? as u64)
    })
    .await
    .map_err(new_indexer_tokio_err)??;

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
            error!("Error polling epoch status {}", e.to_string());
        }
    }

    Ok(())
}

/// Sync data from state server starting from last epoch recorded
/// in database
async fn sync_state(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    mut client: RollupsStateServer,
    postgres_config: &PostgresConfig,
    dapp_address: &Address,
    history_address: &Address,
    input_box_address: &Address,
    confirmations: usize,
) -> Result<(), crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        &postgres_config.postgres_hostname,
        postgres_config.postgres_port,
        &postgres_config.postgres_user,
        &postgres_config.postgres_password,
        &postgres_config.postgres_db,
    )
    .await
    .context(crate::error::TokioError)?;

    // Get last known state server epoch from database
    let db_epoch_index = tokio::task::spawn_blocking(move || {
        Ok(crate::db_service::get_current_db_epoch(
            &conn,
            EpochIndexType::Input,
        )? as u64)
    })
    .await
    .context(crate::error::TokioError)??;

    // Pool epoch state for all previous epochs since the last known epoch
    debug!(
        "Syncing in progress, polling state since epoch {}",
        db_epoch_index
    );
    if let Err(e) = poll_state(
        &message_tx,
        &mut client,
        dapp_address,
        history_address,
        input_box_address,
        confirmations,
    )
    .await
    {
        error!("Error polling epoch status {}", e.to_string());
    }

    Ok(())
}

async fn sync_data(
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    config: Arc<IndexerConfig>,
    health_status: Arc<async_mutex::Mutex<HealthStatus>>,
) -> Result<(), crate::error::Error> {
    // Sync notices from machine manager
    info!("Syncing up to date data from machine manager");

    let db_config = config.database.clone();
    let session_id = config.session_id.clone();
    let mm_endpoint = config.mm_endpoint.clone();

    // Sync inputs and epochs from server manager using backoff strategy
    let sync_server_manager_task = backoff::future::retry(
        backoff::ExponentialBackoff::default(),
        || async {
            let message = format!(
                "Connecting for sync to server manager {}",
                &mm_endpoint
            );
            info!("{}", &message);
            health_status.lock().await.server_manager = Err(message);
            let client = ServerManagerClient::connect(mm_endpoint.to_string())
                .await
                .context(super::error::TonicTransportError)
                .map_err(rollups_data::new_backoff_err)?;

            info!("Trying to sync epoch status from server manager...");
            match sync_epoch_status(
                message_tx.clone(),
                client,
                &db_config,
                &session_id,
            )
            .await
            {
                Ok(()) => {
                    info!("Machine manager sync finished successfully");
                    health_status.lock().await.server_manager = Ok(());
                    Ok(())
                }
                Err(e) => {
                    let err_message = format!(
                        "Failed to sync from server manager, details: {}",
                        e
                    );
                    error!("{}", &err_message);
                    health_status.lock().await.server_manager =
                        Err(err_message);
                    Err(rollups_data::new_backoff_err(e))
                }
            }
        },
    );

    // Sync inputs and epochs from state server using backoff strategy
    let state_server_endpoint = config.state_server_endpoint.clone();
    let sync_state_server_task = backoff::future::retry(
        backoff::ExponentialBackoff::default(),
        || async {
            info!(
                "Connecting for sync to state server {}",
                &state_server_endpoint
            );

            let client = tonic::transport::Channel::from_shared(
                state_server_endpoint.to_owned(),
            )
            .expect(&format!(
                "invalid state-fold-server uri {}",
                state_server_endpoint
            ))
            .connect()
            .await
            .map(GrpcStateFoldClient::new_from_channel)
            .context(super::error::TonicTransportError)
            .map_err(rollups_data::new_backoff_err)?;

            info!("Trying to sync state from state server...");
            match sync_state(
                message_tx.clone(),
                client,
                &config.database,
                &config.dapp_deployment.dapp_address,
                &config.rollups_deployment.history_address,
                &config.rollups_deployment.input_box_address,
                config.confirmations,
            )
            .await
            {
                Ok(()) => {
                    info!("State server sync finished successfully");
                    Ok(())
                }
                Err(e) => {
                    error!(
                        "Failed to sync state server, details: {}",
                        e.to_string()
                    );
                    Err(rollups_data::new_backoff_err(e))
                }
            }
        },
    );

    let (result_server_manager_sync, result_state_server_sync) =
        tokio::join!(sync_server_manager_task, sync_state_server_task);
    result_server_manager_sync?;
    result_state_server_sync?;

    Ok(())
}

/// Create and run new instance of data polling service
pub async fn run(
    config: IndexerConfig,
    message_tx: mpsc::Sender<rollups_data::database::Message>,
    health_status: Arc<async_mutex::Mutex<HealthStatus>>,
) -> Result<(), crate::error::Error> {
    let config = Arc::new(config);
    match sync_data(message_tx.clone(), config.clone(), health_status.clone())
        .await
    {
        Ok(()) => {}
        Err(e) => {
            // If unable to sync, exit indexer
            error!("Failed to sync data: {}", e.to_string());
            return Err(e);
        }
    }

    // Start polling loop
    polling_loop(config, message_tx, health_status).await
}

pub mod testing {
    use super::*;

    pub async fn test_process_epoch_status_response(
        epoch_status_response: GetEpochStatusResponse,
        message_tx: &mpsc::Sender<Message>,
        session_id: &str,
        epoch_index: u64,
    ) -> Result<u64, crate::error::Error> {
        process_epoch_status_response(
            epoch_status_response,
            message_tx,
            session_id,
            epoch_index,
        )
        .await
    }

    pub async fn test_process_state_response(
        rollups_state: Arc<RollupsState>,
        dapp_address: &Address,
        message_tx: &mpsc::Sender<Message>,
    ) -> Result<(), crate::error::Error> {
        process_state_response(rollups_state, dapp_address, message_tx).await
    }
}
