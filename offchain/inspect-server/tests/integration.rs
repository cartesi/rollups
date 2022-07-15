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

use actix_web::dev::ServerHandle;
use std::sync::Arc;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;
use tonic::{transport::Server, Request, Response, Status};

use inspect_server::grpc::cartesi_machine::Void;
use inspect_server::grpc::server_manager::{
    server_manager_server::{ServerManager, ServerManagerServer},
    AdvanceStateRequest, CompletionStatus, EndSessionRequest,
    FinishEpochRequest, GetEpochStatusRequest, GetEpochStatusResponse,
    GetSessionStatusRequest, GetSessionStatusResponse, GetStatusResponse,
    InspectStateRequest, InspectStateResponse, Report, StartSessionRequest,
    StartSessionResponse,
};
use inspect_server::grpc::versioning::GetVersionResponse;
use inspect_server::server::{self, HttpInspectResponse};
use inspect_server::{config::Config, inspect::InspectClient};

const SERVER_MANAGER_ADDRESS: &'static str = "127.0.0.1:50001";
const INSPECT_SERVER_ADDRESS: &'static str = "127.0.0.1:50002";
const SESSION_ID: &'static str = "default session";
const ACTIVE_EPOCH_INDEX: u64 = 123;
const CURRENT_INPUT_INDEX: u64 = 456;

#[tokio::test]
#[serial_test::serial]
async fn test_simple_payload() {
    test_payload("hello", "hello").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_payload_with_spaces() {
    test_payload("hello world", "hello world").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_url_encoded_payload() {
    test_payload("hello%20world", "hello world").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_payload_with_slashes() {
    test_payload("user/123/name", "user/123/name").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_payload_with_empty_payload() {
    test_payload("", "").await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_single_report() {
    let reports = vec![Report {
        payload: vec![1, 2, 3],
    }];
    test_reports(reports).await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_multiple_reports() {
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
    test_reports(reports).await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_exception() {
    let payload = "exception payload message";
    let servers = setup(
        "hello".as_bytes(),
        vec![],
        Some(payload.bytes().collect()),
        CompletionStatus::Exception,
        None,
    )
    .await;
    let response = send_request("hello", 200)
        .await
        .expect("failed to obtain response");
    assert_eq!(response.status, "Exception");
    assert_eq!(response.reports.len(), 0);
    assert_eq!(
        response.exception_payload,
        Some(String::from("0x") + &hex::encode(payload))
    );
    teardown(servers).await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_error_when_server_manager_is_down() {
    let inspect_server = InspectServerWrapper::start(None).await;
    let err_msg = send_request("hello", 502)
        .await
        .expect_err("failed to obtain response");
    assert_eq!(
        err_msg,
        "Failed to connect to server manager: transport error"
    );
    inspect_server.stop().await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_custom_path_prefix() {
    let servers = setup(
        "hello".as_bytes(),
        vec![],
        None,
        CompletionStatus::Accepted,
        Some(String::from("/inspect")),
    )
    .await;
    let response = send_request("inspect/hello", 200)
        .await
        .expect("failed to obtain response");
    assert_eq!(response.status, "Accepted");
    assert_eq!(response.reports.len(), 0);
    assert_eq!(response.exception_payload, None);
    teardown(servers).await;
}

async fn test_payload(sent_payload: &str, expected_payload: &str) {
    let servers = setup(
        expected_payload.as_bytes(),
        vec![],
        None,
        CompletionStatus::Accepted,
        None,
    )
    .await;
    let response = send_request(sent_payload, 200)
        .await
        .expect("failed to obtain response");
    assert_eq!(response.status, "Accepted");
    assert_eq!(response.reports.len(), 0);
    assert_eq!(response.exception_payload, None);
    teardown(servers).await;
}

async fn test_reports(reports: Vec<Report>) {
    let servers = setup(
        "hello".as_bytes(),
        reports.clone(),
        None,
        CompletionStatus::Accepted,
        None,
    )
    .await;
    let response = send_request("hello", 200)
        .await
        .expect("failed to obtain response");
    assert_eq!(response.status, "Accepted");
    assert_eq!(response.exception_payload, None);
    assert_eq!(response.reports.len(), reports.len());
    for (received, expected) in response.reports.iter().zip(reports) {
        let expected_payload =
            String::from("0x") + &hex::encode(expected.payload);
        assert_eq!(received.payload, expected_payload);
    }
    teardown(servers).await;
}

// Send a inspect request to the inspect server.
// Return the inspect response json when the status code is 200.
// Else, return the error message.
async fn send_request(
    payload: &str,
    expected_status: u16,
) -> Result<HttpInspectResponse, String> {
    let url = format!("http://{}/{}", INSPECT_SERVER_ADDRESS, payload);
    let response = reqwest::get(url).await.expect("failed to send inspect");
    assert_eq!(response.status(), expected_status);
    if response.status() == 200 {
        let response = response
            .json::<HttpInspectResponse>()
            .await
            .expect("failed to decode json response");
        assert_eq!(response.metadata.active_epoch_index, ACTIVE_EPOCH_INDEX);
        assert_eq!(response.metadata.current_input_index, CURRENT_INPUT_INDEX);
        Ok(response)
    } else {
        Err(response
            .text()
            .await
            .expect("failed to obtain response body"))
    }
}

async fn setup(
    expected_payload: &[u8],
    returned_reports: Vec<Report>,
    returned_exception: Option<Vec<u8>>,
    returned_completion_status: CompletionStatus,
    path_prefix: Option<String>,
) -> (MockServerManagerWrapper, InspectServerWrapper) {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();
    let server_manager = MockServerManagerWrapper::start(
        expected_payload,
        returned_reports,
        returned_exception,
        returned_completion_status,
    )
    .await;
    let inspect_server = InspectServerWrapper::start(path_prefix).await;
    (server_manager, inspect_server)
}

async fn teardown(
    (server_manager, inspect_server): (
        MockServerManagerWrapper,
        InspectServerWrapper,
    ),
) {
    inspect_server.stop().await;
    server_manager.stop().await;
}

struct InspectServerWrapper {
    server_handle: ServerHandle,
    join_handle: JoinHandle<()>,
}

impl InspectServerWrapper {
    /// Start the inspect server in another thread.
    /// This function blocks until the server is ready.
    async fn start(path_prefix: Option<String>) -> Self {
        let config = Config {
            inspect_server_address: INSPECT_SERVER_ADDRESS.to_string(),
            server_manager_address: SERVER_MANAGER_ADDRESS.to_string(),
            session_id: SESSION_ID.to_string(),
            path_prefix,
        };
        let inspect_client = InspectClient::new(&config);
        let (handle_tx, handle_rx) = oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let server = server::create(&config, inspect_client)
                .expect("failed to start inspect server");
            handle_tx
                .send(server.handle())
                .expect("failed to send server handle");
            server.await.expect("inspect server execution failed");
        });
        let server_handle =
            handle_rx.await.expect("failed to received server handle");
        Self {
            server_handle,
            join_handle,
        }
    }

    /// Stop the inspect server.
    /// This function blocks util the server is shut down.
    async fn stop(self) {
        self.server_handle.stop(true).await;
        self.join_handle
            .await
            .expect("failed to stop inspect server");
    }
}

struct MockServerManagerWrapper {
    shutdown: Arc<Notify>,
    join_handle: JoinHandle<()>,
}

impl MockServerManagerWrapper {
    /// Start the server manager in another thread.
    /// This function blocks until the server is ready.
    async fn start(
        expected_payload: &[u8],
        returned_reports: Vec<Report>,
        returned_exception: Option<Vec<u8>>,
        returned_completion_status: CompletionStatus,
    ) -> Self {
        let expected_request = InspectStateRequest {
            session_id: SESSION_ID.to_string(),
            query_payload: expected_payload.into(),
        };
        let response = InspectStateResponse {
            session_id: SESSION_ID.to_string(),
            active_epoch_index: ACTIVE_EPOCH_INDEX,
            current_input_index: CURRENT_INPUT_INDEX,
            exception_data: returned_exception,
            status: returned_completion_status as i32,
            reports: returned_reports,
        };
        let service = MockServerManager {
            expected_request,
            response,
        };
        let address = SERVER_MANAGER_ADDRESS.parse().expect("invalid address");
        let ready = Arc::new(Notify::new());
        let shutdown = Arc::new(Notify::new());
        let join_handle = {
            let ready = ready.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                let server = Server::builder()
                    .add_service(ServerManagerServer::new(service))
                    .serve_with_shutdown(address, shutdown.notified());
                ready.notify_one();
                server.await.expect("failed to start server manager");
            })
        };
        ready.notified().await;
        Self {
            shutdown,
            join_handle,
        }
    }

    /// Stop the server manager.
    /// This function blocks until the server is shut down.
    async fn stop(self) {
        self.shutdown.notify_one();
        self.join_handle
            .await
            .expect("failed to shutdown server manager");
    }
}

struct MockServerManager {
    expected_request: InspectStateRequest,
    response: InspectStateResponse,
}

#[tonic::async_trait]
impl ServerManager for MockServerManager {
    async fn inspect_state(
        &self,
        request: Request<InspectStateRequest>,
    ) -> Result<Response<InspectStateResponse>, Status> {
        assert_eq!(self.expected_request, request.into_inner());
        Ok(Response::new(self.response.clone()))
    }

    async fn get_version(
        &self,
        _: Request<Void>,
    ) -> Result<Response<GetVersionResponse>, Status> {
        unimplemented!()
    }

    async fn start_session(
        &self,
        _: Request<StartSessionRequest>,
    ) -> Result<Response<StartSessionResponse>, Status> {
        unimplemented!()
    }

    async fn end_session(
        &self,
        _: Request<EndSessionRequest>,
    ) -> Result<Response<Void>, Status> {
        unimplemented!()
    }

    async fn advance_state(
        &self,
        _: Request<AdvanceStateRequest>,
    ) -> Result<Response<Void>, Status> {
        unimplemented!()
    }

    async fn finish_epoch(
        &self,
        _: Request<FinishEpochRequest>,
    ) -> Result<Response<Void>, Status> {
        unimplemented!()
    }

    async fn get_status(
        &self,
        _: Request<Void>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        unimplemented!()
    }

    async fn get_session_status(
        &self,
        _: Request<GetSessionStatusRequest>,
    ) -> Result<Response<GetSessionStatusResponse>, Status> {
        unimplemented!()
    }

    async fn get_epoch_status(
        &self,
        _: Request<GetEpochStatusRequest>,
    ) -> Result<Response<GetEpochStatusResponse>, Status> {
        unimplemented!()
    }
}
