// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_gets_status_with_no_sessions_running() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    let response = grpc_client
        .get_status(grpc_client::Void {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetStatusResponse { session_id: vec![] }
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_gets_status_with_a_single_session_running() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let response = grpc_client
        .get_status(grpc_client::Void {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetStatusResponse {
            session_id: vec![String::from("rollup session")]
        }
    );
}
