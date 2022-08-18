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

mod common;
use crate::common::*;

struct FixedResponseInspect {
    response: MockInspectResponse,
}

#[tonic::async_trait]
impl MockInspect for FixedResponseInspect {
    async fn inspect_state(&self, _: Vec<u8>) -> MockInspectResponse {
        self.response.clone()
    }
}

async fn test_response(sent: MockInspectResponse, expected_status: &str) {
    let mock = FixedResponseInspect {
        response: sent.clone(),
    };
    let state = TestState::setup(mock).await;
    let response = send_request("").await.expect("failed to obtain response");
    assert_eq!(&response.status, expected_status);
    assert_eq!(
        response.exception_payload,
        sent.exception.as_ref().map(hex_to_bin)
    );
    assert_eq!(response.reports.len(), sent.reports.len());
    for (received, sent) in response.reports.iter().zip(sent.reports) {
        assert_eq!(received.payload, hex_to_bin(&sent.payload));
    }
    assert_eq!(response.metadata.active_epoch_index, ACTIVE_EPOCH_INDEX);
    assert_eq!(response.metadata.current_input_index, CURRENT_INPUT_INDEX);
    state.teardown().await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_response_with_no_reports() {
    let response = MockInspectResponse {
        reports: vec![],
        exception: None,
        completion_status: CompletionStatus::Accepted,
    };
    test_response(response, "Accepted").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_response_with_single_report() {
    let response = MockInspectResponse {
        reports: vec![Report {
            payload: vec![1, 2, 3],
        }],
        exception: None,
        completion_status: CompletionStatus::Accepted,
    };
    test_response(response, "Accepted").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_response_with_multiple_reports() {
    let reports = vec![
        Report {
            payload: vec![1, 2, 3],
        },
        Report {
            payload: vec![4, 5, 6],
        },
        Report {
            payload: vec![7, 8, 9],
        },
    ];
    let response = MockInspectResponse {
        reports,
        exception: None,
        completion_status: CompletionStatus::Accepted,
    };
    test_response(response, "Accepted").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_response_with_reports_and_exception() {
    let response = MockInspectResponse {
        reports: vec![Report {
            payload: vec![1, 2, 3],
        }],
        exception: Some(vec![4, 5, 6]),
        completion_status: CompletionStatus::Exception,
    };
    test_response(response, "Exception").await;
}
