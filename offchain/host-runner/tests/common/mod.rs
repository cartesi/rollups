// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

#![allow(dead_code)]

pub mod config;
pub mod grpc_client;
pub mod http_client;
pub mod manager;

pub fn create_address() -> Vec<u8> {
    rand::random::<[u8; 20]>().into()
}

pub fn create_payload() -> Vec<u8> {
    rand::random::<[u8; 16]>().into()
}

pub async fn setup_advance_state(
    grpc_client: &mut grpc_client::ServerManagerClient,
    session_id: &str,
) {
    grpc_client
        .start_session(grpc_client::create_start_session_request(session_id))
        .await
        .unwrap();
    grpc_client
        .advance_state(grpc_client::create_advance_state_request(
            session_id, 0, 0,
        ))
        .await
        .unwrap();
    http_client::finish("accept".into()).await.unwrap();
}

pub async fn finish_advance_state(
    grpc_client: &mut grpc_client::ServerManagerClient,
    session_id: &str,
) -> Option<grpc_client::ProcessedInput> {
    // Send a finish request in a separate thread.
    let handle = tokio::spawn(http_client::finish("accept".into()));

    // Wait for the input to be processed.
    const RETRIES: i32 = 10;
    let mut processed = None;
    for _ in 0..RETRIES {
        processed = grpc_client
            .get_epoch_status(grpc_client::GetEpochStatusRequest {
                session_id: session_id.into(),
                epoch_index: 0,
            })
            .await
            .unwrap()
            .into_inner()
            .processed_inputs
            .pop();
        if processed.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Wait for the finish to return.
    // It should return error because there isn't a next request to be processed.
    handle
        .await
        .expect("tokio spawn failed")
        .expect_err("finish should return error");

    processed
}
