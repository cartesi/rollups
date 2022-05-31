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
use ethers::core::types::{Address, U256};
use offchain::fold::types::{
    AccumulatingEpoch, EpochWithClaims, Input, PhaseState, RollupsState,
};
use rollups_data::database::{
    DbInput, DbNotice, DbProof, DbReport, DbVoucher, Message,
};
use snafu::ResultExt;
use state_fold::types::BlockState;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
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

    info!("Poll epoch status: polling for epoch {}", epoch_index);
    let epoch_status_response =
        get_epoch_status(client, session_id, epoch_index).await?;

    for input in epoch_status_response.processed_inputs {
        // Process reports
        for (rindex, report) in input.reports.iter().enumerate() {
            trace!("Poll epoch status: sending report with session id {}, epoch_index {} input_index {} report_index {} report {:?}",
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
                    "Poll epoch status: error passing report message to db {}",
                    e.to_string()
                )
            }
        }

        if let Some(one_of) = input.processed_oneof {
            match one_of {
                cartesi_server_manager::processed_input::ProcessedOneof::Result(input_result) => {
                    // Process vouchers
                    for (vindex, voucher) in input_result.vouchers.iter().enumerate() {
                        // Send one voucher to database service
                        trace!("Poll epoch status: sending voucher with session id {}, epoch_index {} input_index {} voucher_index {} voucher {:?}",
                                session_id, epoch_index, input.input_index, &vindex, voucher);

                        // Construct and send notice proof
                        let proof = DbProof {
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
                                .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect(),
                            output_hashes_in_epoch_siblings:    input.voucher_hashes_in_epoch
                                .as_ref()
                                .unwrap_or(&Default::default())
                                .sibling_hashes
                                .iter()
                                .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect()
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
                                    .unwrap_or(&cartesi_server_manager::Address { data: vec![] })
                                    .data,
                            ).as_str(),
                            // Payload is in raw format
                            payload: Some(voucher.payload.clone())
                        })).await {
                            error!("Poll epoch status: error passing voucher message to db {}", e.to_string())
                        }
                    }
                    // Process notices
                    for (nindex, notice) in input_result.notices.iter().enumerate() {
                        // Send one notice to database service
                        trace!("Poll epoch status: sending notice with session id {}, epoch_index {} input_index {} notice_index {}",
                                session_id, epoch_index, input.input_index, &nindex);

                        // Construct and send notice proof
                        let proof = DbProof {
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
                                .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect(),
                            output_hashes_in_epoch_siblings:    input.notice_hashes_in_epoch
                                .as_ref()
                                .unwrap_or(&Default::default())
                                .sibling_hashes
                                .iter()
                                .map(|hash| "0x".to_string() + hex::encode(&hash.data).as_str()).collect()
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
                            error!("Poll epoch status: error passing notice message to db {}", e.to_string())
                        }
                    }
                },
                cartesi_server_manager::processed_input::ProcessedOneof::SkipReason(reason) => {
                    info!("Poll epoch status: skip processed input for reason {:?}", reason);
                }
            }
        }
    }
    Ok(epoch_index)
}

/// Get state server current state
/// Process inputs ad send to db service
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
        "Poll state: json state server retrieve initial state argument: {:?}",
        &json_initial_state
    );

    let state_str = get_state(client, &json_initial_state).await?;
    trace!(
        "Poll state: state received for initial state {:?} is {:?}",
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
                tx_hash: None,
                payload: (*input.payload).clone(),
                timestamp: chrono::NaiveDateTime::from_timestamp(
                    input.timestamp.low_u64() as i64,
                    0,
                ),
            }))
            .await
        {
            error!(
                "Poll state: error passing input message to db {}",
                e.to_string()
            )
        }
    }

    // Process first inputs from finalized epochs
    for finalized_epoch in block_state.state.finalized_epochs.finalized_epochs {
        for (index, input) in finalized_epoch.inputs.inputs.iter().enumerate() {
            debug!("Poll state: processing finalized input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
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

    async fn process_accumulating_epoch(
        message_tx: &mpsc::Sender<Message>,
        accumulating_epoch: AccumulatingEpoch,
    ) {
        for (index, input) in
            accumulating_epoch.inputs.inputs.iter().enumerate()
        {
            debug!("Poll state: processing accumulating epoch input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                    &input.sender, accumulating_epoch.epoch_number.as_u32(), &input.block_number, &input.timestamp, &input.payload );
            send_input(
                index,
                accumulating_epoch.epoch_number.as_u32() as i32,
                input,
                message_tx,
            )
            .await;
        }
    }

    async fn process_epoch_with_claims(
        message_tx: &mpsc::Sender<Message>,
        claimed_epoch: EpochWithClaims,
    ) {
        for (index, input) in claimed_epoch.inputs.inputs.iter().enumerate() {
            debug!("Poll state: processing claimed epoch input with sender: {} epoch {} block_number: {} timestamp: {} payload {:?}",
                    &input.sender, claimed_epoch.epoch_number.as_u32(), &input.block_number, &input.timestamp, &input.payload );
            send_input(
                index,
                claimed_epoch.epoch_number.as_u32() as i32,
                input,
                message_tx,
            )
            .await;
        }
    }

    // Check for current phase state, process inputs accordingly
    info!(
        "Poll state: pooling state server, current rollups phase is {:?}",
        block_state.state.current_phase
    );
    match block_state.state.current_phase {
        PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch } => {
            debug!(
                "Poll state: processing sealed epoch {} while awaiting first claim",
                sealed_epoch.epoch_number
            );
            process_accumulating_epoch(message_tx, sealed_epoch).await;
        }
        PhaseState::InputAccumulation {} => {}
        PhaseState::AwaitingConsensusNoConflict { claimed_epoch }
        | PhaseState::ConsensusTimeout { claimed_epoch }
        | PhaseState::AwaitingDispute { claimed_epoch } => {
            debug!(
                "Poll state: processing claimed epoch {}",
                claimed_epoch.epoch_number
            );
            process_epoch_with_claims(message_tx, claimed_epoch).await;
        }
        PhaseState::AwaitingConsensusAfterConflict {
            claimed_epoch,
            challenge_period_base_ts,
        } => {
            debug!(
                "Poll state: processing claimed epoch {}, challenge period base ts {}",
                claimed_epoch.epoch_number, challenge_period_base_ts
            );
            process_epoch_with_claims(message_tx, claimed_epoch).await;
        }
    }

    // Process current epoch
    debug!(
        "Poll state: processing current epoch {}",
        block_state.state.current_epoch.epoch_number
    );
    process_accumulating_epoch(message_tx, block_state.state.current_epoch)
        .await;

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
    let last_epoch_status_index: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    loop {
        for task in tasks.iter_mut() {
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
                                        Ok(client) => client,
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            error!("Failed to connect to server manager endpoint {}: {}", &config.mm_endpoint, e.to_string());
                                            return (); // return from spanned background task
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
                                                    error!("Error pooling epoch status for epoch {}: {}", epoch_index, e.to_string());
                                                }
                                            }
                                            *last_epoch_index = new_epoch_index;
                                        }
                                    }
                                    Err(e) => {
                                        let last_epoch_index =
                                            *last_epoch_index.lock().await;
                                        error!(
                                            "Error pooling epoch status for epoch index {}: {}", last_epoch_index, e.to_string()
                                        );
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
    postgres_endpoint: &str,
    session_id: &str,
) -> Result<(), crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        postgres_endpoint.into(),
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
    postgres_endpoint: &str,
    dapp_contract_address: Address,
) -> Result<(), crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        postgres_endpoint.into(),
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
    info!("Syncing up to date data from machine manager");
    let client =
        connect_to_server_manager_with_retry(&config.mm_endpoint).await?;
    sync_epoch_status(
        message_tx.clone(),
        client,
        &postgres_endpoint,
        &config.session_id,
    )
    .await?;
    info!("Machine manager sync finished successfully");

    // Sync inputs and epochs from state server
    info!("Syncing up to date data from state server");
    let client =
        connect_to_state_server_with_retry(&config.state_server_endpoint)
            .await?;
    sync_state(
        message_tx.clone(),
        client,
        &postgres_endpoint,
        config.dapp_contract_address,
    )
    .await?;
    info!("State server sync finished successfully");

    Ok(())
}

/// Create and run new instance of data polling service
pub async fn run(
    config: IndexerConfig,
    message_tx: mpsc::Sender<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    let config = Arc::new(config);
    match sync_data(message_tx.clone(), config.clone()).await {
        Ok(()) => {}
        Err(e) => {
            error!("Failed to sync data: {}", e.to_string());
        }
    }

    // Start polling loop
    polling_loop(config, message_tx).await
}
