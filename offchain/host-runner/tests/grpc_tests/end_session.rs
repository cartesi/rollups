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
