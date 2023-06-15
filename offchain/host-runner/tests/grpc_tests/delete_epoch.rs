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

use serial_test::serial;
use tonic::Code;

use crate::common::{
    grpc_client::{self, DeleteEpochRequest, FinishEpochRequest},
    manager,
};

const SESSION_ID: &str = "rollup session";

#[tokio::test]
#[serial]
async fn test_it_fails_to_delete_active_epoch() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .unwrap();

    let err = grpc_client
        .delete_epoch(DeleteEpochRequest {
            epoch_index: 0,
            session_id: SESSION_ID.into(),
        })
        .await
        .expect_err("should fail to delete epoch");

    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
#[serial]
async fn test_it_fails_to_delete_unexisting_epoch() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .expect("should start session");

    let err = grpc_client
        .delete_epoch(DeleteEpochRequest {
            epoch_index: 1,
            session_id: SESSION_ID.into(),
        })
        .await
        .expect_err("should fail to delete epoch");

    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
#[serial]
async fn test_it_fails_to_delete_when_there_is_no_session() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;

    let err = grpc_client
        .delete_epoch(DeleteEpochRequest {
            epoch_index: 1,
            session_id: SESSION_ID.into(),
        })
        .await
        .expect_err("should fail to delete epoch");

    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
#[serial]
async fn test_it_deletes_epoch() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    grpc_client
        .start_session(grpc_client::create_start_session_request(
            "rollup session",
        ))
        .await
        .expect("should start session");

    grpc_client
        .finish_epoch(FinishEpochRequest {
            session_id: SESSION_ID.into(),
            active_epoch_index: 0,
            processed_input_count_within_epoch: 0,
            storage_directory: "".into(),
        })
        .await
        .expect("should finish epoch");

    let response = grpc_client
        .delete_epoch(DeleteEpochRequest {
            epoch_index: 0,
            session_id: SESSION_ID.into(),
        })
        .await;
    assert!(response.is_ok());

    let err = grpc_client
        .get_epoch_status(grpc_client::GetEpochStatusRequest {
            epoch_index: 0,
            session_id: SESSION_ID.into(),
        })
        .await
        .expect_err("epoch should have been deleted");

    assert_eq!(err.code(), Code::InvalidArgument);
}
