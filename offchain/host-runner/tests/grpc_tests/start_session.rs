// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_starts_session_with_new_valid_session() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    let response = grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(response, grpc_client::StartSessionResponse { config: None });
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_start_session_with_same_name() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let err = grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_start_multiple_sessions_because_it_supports_only_one()
{
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session 1",
        ))
        .await
        .unwrap();
    let err = grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session 2",
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}
