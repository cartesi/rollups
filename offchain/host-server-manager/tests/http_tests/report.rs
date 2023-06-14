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
async fn test_it_insert_report_during_advance_state() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    // Send reports
    const N: usize = 3;
    for _ in 0..N {
        http_client::insert_report(http_client::create_payload())
            .await
            .unwrap();
    }
    // Check if reports arrived
    let processed = finish_advance_state(&mut grpc_client, "rollup session")
        .await
        .unwrap();
    assert_eq!(processed.reports.len(), N);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_insert_report_during_inspect_state() {
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
    // Send a reports
    const N: usize = 3;
    for _ in 0..N {
        http_client::insert_report(http_client::create_payload())
            .await
            .unwrap();
    }
    // Perform final finish call
    tokio::spawn(http_client::finish("accept".into()));
    // Obtain the inspect result
    let response = inspect_handle.await.unwrap();
    assert_eq!(response.reports.len(), N);
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_insert_report_with_incorrect_data() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    setup_advance_state(&mut grpc_client, "rollup session").await;
    let response = http_client::insert_report("deadbeef".into()).await;
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
async fn test_it_fails_to_insert_report_during_idle_state() {
    let _manager = manager::Wrapper::new().await;
    // Don't perform setup on purpose
    let response = http_client::insert_report(http_client::create_payload()).await;
    assert_eq!(
        response,
        Err(http_client::HttpError {
            status: 400,
            message: "invalid request report in idle state".into(),
        })
    );
}
