// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_insert_voucher_during_advance_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    // Send vouchers
    const N: usize = 3;
    for i in 0..N {
        let destination = http_client::create_address();
        let payload = http_client::create_payload();
        let result = http_client::insert_voucher(destination, payload)
            .await
            .unwrap();
        assert_eq!(result.index, i);
    }
    // Check if vouchers arrived
    let processed = finish_advance_state(&mut grpc_client, "rollup session")
        .await
        .unwrap();
    match processed.processed_input_one_of.unwrap() {
        grpc_client::processed_input::ProcessedInputOneOf::AcceptedData(
            result,
        ) => {
            assert_eq!(result.vouchers.len(), N);
        }
        grpc_client::processed_input::ProcessedInputOneOf::ExceptionData(_) => {
            panic!("unexpected advance result");
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_insert_voucher_with_incorrect_data() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let response = http_client::insert_voucher(
        http_client::create_address(),
        "deadbeef".into(),
    )
    .await;
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
async fn test_it_fails_to_insert_voucher_during_idle_state() {
    let _manager = manager::Wrapper::new().await;
    // Don't perform setup on purpose
    let destination = http_client::create_address();
    let payload = http_client::create_payload();
    let response = http_client::insert_voucher(destination, payload).await;
    assert_eq!(
        response,
        Err(http_client::HttpError {
            status: 400,
            message: "invalid request voucher in idle state".into(),
        })
    );
}
