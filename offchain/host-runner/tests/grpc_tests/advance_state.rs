// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_advances_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let response = grpc_client
        .advance_state(grpc_client::create_advance_state_request(
            "rollup session",
            0,
            0,
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(response, grpc_client::Void {});
    // Check if state changed to advance with HTTP finish
    let response = http_client::finish("accept".into()).await.unwrap();
    assert!(matches!(
        response,
        http_client::RollupHttpRequest::Advance { .. }
    ));
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_advance_request_with_wrong_parameters() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let invalid_requests = vec![
        // Wrong session id
        grpc_client::create_advance_state_request("rollup session 1", 0, 0),
        // Wrong epoch number
        grpc_client::create_advance_state_request("rollup session", 123, 0),
        // Wrong input index
        grpc_client::create_advance_state_request("rollup session", 0, 123),
    ];
    for request in invalid_requests {
        let err = grpc_client.advance_state(request).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
}
