// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_get_epoch_status_of_empty_epoch() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let response = grpc_client
        .get_epoch_status(grpc_client::GetEpochStatusRequest {
            session_id: "rollup session".into(),
            epoch_index: 0,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetEpochStatusResponse {
            session_id: "rollup session".into(),
            epoch_index: 0,
            state: grpc_client::EpochState::Active as i32,
            processed_inputs: vec![],
            pending_input_count: 0,
            taint_status: None,
        }
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_get_epoch_status_of_epoch_with_voucher_notice_and_report() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let destination = String::from("0x") + &"fa".repeat(20);
    http_client::insert_voucher(destination, "0xdeadbeef".into())
        .await
        .unwrap();
    http_client::insert_notice("0xdeadbeef".into())
        .await
        .unwrap();
    http_client::insert_report("0xdeadbeef".into())
        .await
        .unwrap();
    finish_advance_state(&mut grpc_client, "rollup session").await;
    let response = grpc_client
        .get_epoch_status(grpc_client::GetEpochStatusRequest {
            session_id: "rollup session".into(),
            epoch_index: 0,
        })
        .await
        .unwrap()
        .into_inner();
    let expected = grpc_client::GetEpochStatusResponse {
        session_id: "rollup session".into(),
        epoch_index: 0,
        state: grpc_client::EpochState::Active as i32,
        processed_inputs: vec![grpc_client::ProcessedInput {
            input_index: 0,
            reports: vec![grpc_client::Report {
                payload: vec![222, 173, 190, 239],
            }],
            status: grpc_client::CompletionStatus::Accepted as i32,
            processed_input_one_of: Some(
                grpc_client::processed_input::ProcessedInputOneOf::AcceptedData(
                    grpc_client::AcceptedData {
                        vouchers: vec![grpc_client::Voucher {
                            destination: Some(grpc_client::Address {
                                data: vec![
                                    250, 250, 250, 250, 250, 250, 250, 250,
                                    250, 250, 250, 250, 250, 250, 250, 250,
                                    250, 250, 250, 250,
                                ],
                            }),
                            payload: vec![222, 173, 190, 239],
                        }],
                        notices: vec![grpc_client::Notice {
                            payload: vec![222, 173, 190, 239],
                        }],
                    },
                ),
            ),
        }],
        pending_input_count: 0,
        taint_status: None,
    };
    assert_eq!(response, expected);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_get_non_existent_epoch_status() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let err = grpc_client
        .get_epoch_status(grpc_client::GetEpochStatusRequest {
            session_id: "rollup session".into(),
            epoch_index: 123,
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}
