// Copyright 2022 Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use crate::common::*;

#[tokio::test]
#[serial_test::serial]
async fn test_it_notifies_exception_during_advance_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let payload = create_payload();
    http_client::notify_exception(http_client::convert_binary_to_hex(&payload))
        .await
        .unwrap();
    let processed = finish_advance_state(&mut grpc_client, "rollup session")
        .await
        .unwrap();
    match processed.processed_input_one_of.unwrap() {
        grpc_client::processed_input::ProcessedInputOneOf::AcceptedData(_) => {
            panic!("unexpected advance result");
        }
        grpc_client::processed_input::ProcessedInputOneOf::ExceptionData(result) => {
            assert_eq!(result, payload);
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_notifies_exception_during_inspect_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request("rollup session"))
        .await
        .unwrap();
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
    http_client::finish("accept".into()).await.unwrap();
    let payload = create_payload();
    http_client::notify_exception(http_client::convert_binary_to_hex(&payload))
        .await
        .unwrap();
    let response = inspect_handle.await.unwrap();
    let expected = grpc_client::InspectStateResponse {
        session_id: String::from("rollup session"),
        active_epoch_index: 0,
        processed_input_count: 0,
        status: grpc_client::CompletionStatus::Exception as i32,
        exception_data: Some(payload),
        reports: vec![],
    };
    assert_eq!(response, expected);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_notify_exception_with_incorrect_data() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let response = http_client::notify_exception("deadbeef".into()).await;
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
async fn test_it_fails_to_notify_exception_during_idle_state() {
    let _manager = manager::Wrapper::new().await;
    let response = http_client::notify_exception(http_client::create_payload()).await;
    assert_eq!(
        response,
        Err(http_client::HttpError {
            status: 400,
            message: "invalid request exception in idle state".into(),
        })
    );
}
