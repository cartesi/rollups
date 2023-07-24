// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_insert_notice_during_advance_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    // Send notices
    const N: usize = 3;
    for i in 0..N {
        let result = http_client::insert_notice(http_client::create_payload())
            .await
            .unwrap();
        assert_eq!(result.index, i);
    }
    // Check if notices arrived
    let processed = finish_advance_state(&mut grpc_client, "rollup session")
        .await
        .unwrap();
    match processed.processed_input_one_of.unwrap() {
        grpc_client::processed_input::ProcessedInputOneOf::AcceptedData(
            result,
        ) => {
            assert_eq!(result.notices.len(), N);
        }
        grpc_client::processed_input::ProcessedInputOneOf::ExceptionData(_) => {
            panic!("unexpected advance result");
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_insert_notice_with_incorrect_data() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let response = http_client::insert_notice("deadbeef".into()).await;
    assert_eq!(
        response,
        Err(http_client::HttpError {
            status: 400,
            message: "Failed to decode ethereum binary string deadbeef (expected 0x prefix)".into(),
        })
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_insert_notice_during_idle_state() {
    let _manager = manager::Wrapper::new().await;
    // Don't perform setup on purpose
    let response =
        http_client::insert_notice(http_client::create_payload()).await;
    assert_eq!(
        response,
        Err(http_client::HttpError {
            status: 400,
            message: "invalid request notice in idle state".into(),
        })
    );
}
