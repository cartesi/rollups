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
async fn test_it_gets_version() {
    let _manager = manager::Wrapper::new().await;
    let mut grpc_client = grpc_client::connect().await;
    let response = grpc_client
        .get_version(grpc_client::Void {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response,
        grpc_client::GetVersionResponse {
            version: Some(grpc_client::SemanticVersion {
                major: 0,
                minor: 2,
                patch: 0,
                pre_release: String::from(""),
                build: String::from("host-runner"),
            })
        }
    );
}
