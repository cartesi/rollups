// Copyright Cartesi Pte. Ltd.
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
async fn test_it_gets_session_status_with_no_advance_requests() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    let response = grpc_client
        .get_session_status(grpc_client::GetSessionStatusRequest {
            session_id: "rollup session".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetSessionStatusResponse {
            session_id: "rollup session".into(),
            active_epoch_index: 0,
            epoch_index: vec![0],
            taint_status: None,
        }
    )
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_gets_session_with_multiple_epochs() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();
    const N: u64 = 10;
    for i in 0..N {
        grpc_client
            .finish_epoch(grpc_client::FinishEpochRequest {
                session_id: "rollup session".into(),
                active_epoch_index: i,
                processed_input_count_within_epoch: 0,
                storage_directory: "".into(),
            })
            .await
            .unwrap();
    }
    let response = grpc_client
        .get_session_status(grpc_client::GetSessionStatusRequest {
            session_id: "rollup session".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetSessionStatusResponse {
            session_id: "rollup session".into(),
            active_epoch_index: N,
            epoch_index: (0..N + 1).collect::<Vec<_>>(),
            taint_status: None,
        }
    )
}

#[tokio::test]
#[serial_test::serial]
async fn test_it_fails_to_get_session_status_of_unexistent_session() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    let err = grpc_client
        .get_session_status(grpc_client::GetSessionStatusRequest {
            session_id: "rollup session".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}
