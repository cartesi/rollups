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

use actix_cors::Cors;
use actix_web::{
    dev::Server, error, middleware, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

use crate::config::InspectServerConfig;
use crate::error::InspectError;
use crate::inspect::{
    CompletionStatus, InspectClient, InspectStateResponse, Report,
};

pub fn create(
    config: &InspectServerConfig,
    inspect_client: InspectClient,
) -> std::io::Result<Server> {
    let inspect_path = config.inspect_path_prefix.clone() + "/{payload:.*}";
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(inspect_client.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(
                web::resource(inspect_path.clone())
                    .route(web::get().to(inspect)),
            )
    })
    .bind(config.inspect_server_address.clone())?
    .run();
    Ok(server)
}

async fn inspect(
    request: HttpRequest,
    payload: web::Path<String>,
    inspect_client: web::Data<InspectClient>,
) -> actix_web::error::Result<impl Responder> {
    let mut payload = payload.into_inner();
    if let Some(query) = request.uri().query() {
        payload = payload + "?" + query;
    }
    let payload = payload.as_bytes().to_vec();
    let response = inspect_client.inspect(payload).await?;
    let http_response = HttpInspectResponse::from(response);
    Ok(HttpResponse::Ok().json(http_response))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpInspectResponse {
    pub status: String,
    pub exception_payload: Option<String>,
    pub reports: Vec<HttpReport>,
    pub processed_input_count: u64,
}

impl From<InspectStateResponse> for HttpInspectResponse {
    fn from(response: InspectStateResponse) -> HttpInspectResponse {
        let reports =
            response.reports.into_iter().map(HttpReport::from).collect();
        HttpInspectResponse {
            status: convert_status(response.status),
            exception_payload: response.exception_data.map(hex_encode),
            reports,
            processed_input_count: response.processed_input_count,
        }
    }
}

fn convert_status(status: i32) -> String {
    // Unfortunaly, the gRPC interface uses i32 instead of a Enum type,
    // so it is clearer to use if-else instead of match.
    if status == CompletionStatus::Accepted as i32 {
        String::from("Accepted")
    } else if status == CompletionStatus::Rejected as i32 {
        String::from("Rejected")
    } else if status == CompletionStatus::Exception as i32 {
        String::from("Exception")
    } else if status == CompletionStatus::MachineHalted as i32 {
        String::from("MachineHalted")
    } else if status == CompletionStatus::CycleLimitExceeded as i32 {
        String::from("CycleLimitExceeded")
    } else if status == CompletionStatus::TimeLimitExceeded as i32 {
        String::from("TimeLimitExceeded")
    } else {
        log::error!("Invalid status received from server-manager: {}", status);
        String::from("Unknown")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpReport {
    pub payload: String,
}

impl From<Report> for HttpReport {
    fn from(report: Report) -> HttpReport {
        HttpReport {
            payload: hex_encode(report.payload),
        }
    }
}

fn hex_encode(payload: Vec<u8>) -> String {
    String::from("0x") + &hex::encode(payload)
}

impl From<InspectError> for error::Error {
    fn from(e: InspectError) -> error::Error {
        log::warn!("{}", e.to_string());
        match e {
            InspectError::FailedToConnect { .. } => {
                error::ErrorBadGateway(e.to_string())
            }
            InspectError::InspectFailed { .. } => {
                error::ErrorBadRequest(e.to_string())
            }
            _ => error::ErrorBadGateway(e.to_string()),
        }
    }
}
