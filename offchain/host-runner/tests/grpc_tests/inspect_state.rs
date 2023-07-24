// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_inspects_and_receive_a_report() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    // Send the inspect request in a separate thread
    let inspect_handle = tokio::spawn(async move {
        grpc_client
            .inspect_state(grpc_client::InspectStateRequest {
                session_id: "rollup session".into(),
                query_payload: create_payload(),
            })
            .await
            .unwrap()
            .into_inner()
    });
    // Send HTTP requests
    while let Err(e) = http_client::finish("accept".into()).await {
        assert_eq!(e.status, 202);
    }
    let payload = create_payload();
    http_client::insert_report(http_client::convert_binary_to_hex(&payload))
        .await
        .unwrap();
    http_client::finish("accept".into()).await.unwrap_err();
    // Obtain the inspect response and check it
    let response = inspect_handle.await.unwrap();
    let expected = grpc_client::InspectStateResponse {
        session_id: String::from("rollup session"),
        active_epoch_index: 0,
        processed_input_count: 0,
        status: grpc_client::CompletionStatus::Accepted as i32,
        exception_data: None,
        reports: vec![grpc_client::Report { payload }],
    };
    assert_eq!(response, expected);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_reports_session_state_correctly() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;

    // Send an input and finish the first epoch
    setup_advance_state(&mut grpc_client, "rollup session").await;
    finish_advance_state(&mut grpc_client, "rollup session").await;
    grpc_client
        .finish_epoch(grpc_client::FinishEpochRequest {
            session_id: "rollup session".into(),
            active_epoch_index: 0,
            processed_input_count_within_epoch: 1,
            storage_directory: "".into(),
        })
        .await
        .expect("should finish epoch");

    // Send an inspect request in the second epoch
    let inspect_handle = tokio::spawn(async move {
        grpc_client
            .inspect_state(grpc_client::InspectStateRequest {
                session_id: "rollup session".into(),
                query_payload: create_payload(),
            })
            .await
            .unwrap()
            .into_inner()
    });

    // Get inspect request state request
    http_client::finish("accept".into()).await.unwrap();

    // Accept inspect request
    http_client::finish("accept".into()).await.unwrap_err();

    let response = inspect_handle.await.unwrap();
    let expected = grpc_client::InspectStateResponse {
        session_id: String::from("rollup session"),
        active_epoch_index: 1,
        processed_input_count: 1,
        status: grpc_client::CompletionStatus::Accepted as i32,
        exception_data: None,
        reports: vec![],
    };
    assert_eq!(response, expected);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_inspect_state_concurrently() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    // Send the inspect request in a separate thread
    let inspect_handle = {
        let mut grpc_client = grpc_client.clone();
        tokio::spawn(async move {
            grpc_client
                .inspect_state(grpc_client::InspectStateRequest {
                    session_id: "rollup session".into(),
                    query_payload: create_payload(),
                })
                .await
                .unwrap()
                .into_inner()
        })
    };
    // Wait until the first request starts to be processed
    while let Err(e) = http_client::finish("accept".into()).await {
        assert_eq!(e.status, 202);
    }
    // Send second inspect request
    let status = grpc_client
        .inspect_state(grpc_client::InspectStateRequest {
            session_id: "rollup session".into(),
            query_payload: create_payload(),
        })
        .await
        .unwrap_err();
    assert_eq!(status.code(), tonic::Code::Aborted);
    assert_eq!(status.message(), "concurrent call in session");
    // Finish first inspect request
    http_client::finish("accept".into()).await.unwrap_err();
    inspect_handle.await.unwrap();
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_queue_inspect_during_advance_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    // Start advance state
    setup_advance_state(&mut grpc_client, "rollup session").await;
    // Send the inspect request in a separate thread
    let inspect_handle = {
        let mut grpc_client = grpc_client.clone();
        tokio::spawn(async move {
            grpc_client
                .inspect_state(grpc_client::InspectStateRequest {
                    session_id: "rollup session".into(),
                    query_payload: create_payload(),
                })
                .await
                .unwrap()
                .into_inner()
        })
    };
    // Finish advance request and start inspect request
    http_client::finish("accept".into()).await.unwrap();
    // Finish inspect request
    http_client::finish("accept".into()).await.unwrap_err();
    inspect_handle.await.unwrap();
}
