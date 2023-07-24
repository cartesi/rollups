// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_ends_existing_session() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    grpc_client
        .end_session(grpc_client::EndSessionRequest {
            session_id: "rollup session".into(),
        })
        .await
        .unwrap();
    let err = grpc_client
        .end_session(grpc_client::EndSessionRequest {
            session_id: "rollup session".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_ends_non_existing_session() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session 1",
        ))
        .await
        .unwrap();
    let err = grpc_client
        .end_session(grpc_client::EndSessionRequest {
            session_id: "rollup session 2".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}
