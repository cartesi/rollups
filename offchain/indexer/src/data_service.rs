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

use crate::config::IndexerConfig;
use crate::db_service::{Message, NoticeInfo};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

use crate::grpc::{
    cartesi_server_manager,
    cartesi_server_manager::{
        server_manager_client::ServerManagerClient, GetEpochStatusRequest,
    },
};

async fn poll_epoch_status(
    config: Arc<IndexerConfig>,
    message_tx: mpsc::Sender<Message>,
    mut client: ServerManagerClient<tonic::transport::Channel>,
) -> Result<(), crate::error::Error> {
    debug!("Polling epoch status");
    let request = GetEpochStatusRequest {
        session_id: config.session_id.clone(),
        epoch_index: 0, //todo fix
    };
    let response = client
        .get_epoch_status(tonic::Request::new(request.clone()))
        .await
        .map_err(|e| crate::error::Error::TonicStatusError { source: e })?
        .into_inner();

    for input in response.processed_inputs {
        debug!(
            "Processed input {} reports number {}",
            input.input_index,
            input.reports.len()
        );
        for one_of in input.processed_oneof {
            match one_of {
                cartesi_server_manager::processed_input::ProcessedOneof::Result(input_result) => {
                    for (nindex, notice) in input_result.notices.iter().enumerate() {
                        // Send one notice
                        trace!("Sending notice with session id {}, epoch_index {} input_index {} notice_index {}",
                                &request.session_id, &request.epoch_index, input.input_index, &nindex);
                        if let Err(e) = message_tx.send(Message::Notice(NoticeInfo {
                            session_id: request.session_id.clone(),
                            epoch_index: request.epoch_index,
                            input_index: input.input_index,
                            notice_index: nindex as u64
                        }, notice.clone())).await {
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

enum TaskType {
    GetEpochStatus,
}

struct Task {
    interval: std::time::Duration,
    next_execution: std::time::SystemTime,
    task_type: TaskType,
}

async fn polling_loop(
    config: IndexerConfig,
    message_tx: mpsc::Sender<Message>,
) -> Result<(), crate::error::Error> {
    let config = Arc::new(config);
    let loop_interval = std::time::Duration::from_millis(100);
    let poll_interval = std::time::Duration::from_secs(config.interval);

    let mut tasks = vec![Task {
        interval: poll_interval,
        next_execution: std::time::SystemTime::now().add(poll_interval),
        task_type: TaskType::GetEpochStatus,
    }];

    // Pooling tasks loop
    info!("Starting data pooling loop");
    loop {
        for task in tasks.iter_mut() {
            if task.next_execution <= std::time::SystemTime::now() {
                match task.task_type {
                    TaskType::GetEpochStatus => {
                        debug!("Performing get epoch status");
                        {
                            let config = config.clone();
                            let message_tx = message_tx.clone();
                            tokio::spawn(async move {
                                debug!("Performing get epoch status from client {}", &config.mm_endpoint);
                                // Connect to server manager client
                                let server_manager_client =
                                    match ServerManagerClient::connect(
                                        config.mm_endpoint.clone(),
                                    )
                                    .await
                                    {
                                        Ok(client) => client,
                                        Err(e) => {
                                            // In case of error, continue with the loop, try to connect again after interval
                                            error!("Failed to connect to server manager endpoint {}: {}", &config.mm_endpoint, e.to_string());
                                            return ();
                                        }
                                    };

                                if let Err(e) = poll_epoch_status(
                                    config,
                                    message_tx,
                                    server_manager_client,
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

/// Create and run new instance of db service
pub async fn run(
    config: IndexerConfig,
    message_tx: mpsc::Sender<Message>,
) -> Result<(), crate::error::Error> {
    polling_loop(config, message_tx).await
}
