// Copyright 2023 Cartesi Pte. Ltd.
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

use json::{object, JsonValue};

async fn print_response<T: hyper::body::HttpBody>(
    response: hyper::Response<T>,
    endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    <T as hyper::body::HttpBody>::Error: 'static,
    <T as hyper::body::HttpBody>::Error: std::error::Error,
{
    let response_status = response.status().as_u16();
    let response_body = hyper::body::to_bytes(response).await?;
    tracing::info!(
        "Received {} status {} body {}",
        endpoint,
        response_status,
        std::str::from_utf8(&response_body)?
    );
    Ok(())
}

async fn handle_advance(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    tracing::info!("Received advance request data {}", &request);
    let payload = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    tracing::info!("Adding notice");
    let notice = object! {"payload" => payload.clone()};
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(format!("{}/notice", server_addr))
        .body(hyper::Body::from(notice.dump()))?;
    let response = client.request(req).await?;
    print_response(response, "notice").await?;

    let rollup_address = request["data"]["metadata"]["msg_sender"]
        .as_str()
        .ok_or("Missing msg_sender")?;
    tracing::info!("Adding voucher");
    let voucher = object! { "address" => rollup_address.clone(), "payload" => payload.clone()};
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(format!("{}/voucher", server_addr))
        .body(hyper::Body::from(voucher.dump()))?;
    let response = client.request(req).await?;
    print_response(response, "voucher").await?;

    Ok("accept")
}

async fn handle_inspect(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    tracing::info!("Received inspect request data {}", &request);
    let payload = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    tracing::info!("Adding report");
    let report = object! {"payload" => payload.clone()};
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(format!("{}/report", server_addr))
        .body(hyper::Body::from(report.dump()))?;
    let response = client.request(req).await?;
    print_response(response, "report").await?;
    Ok("accept")
}

pub struct EchoDAppFixture {}

impl EchoDAppFixture {
    async fn start_echo_dapp(
        server_addr: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = hyper::Client::new();

        let mut status = "accept";
        loop {
            tracing::info!("Sending finish");

            let response = object! {"status" => status.clone()};
            let request = hyper::Request::builder()
                .method(hyper::Method::POST)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .uri(format!("{}/finish", &server_addr))
                .body(hyper::Body::from(response.dump()))?;
            let response = client.request(request).await?;
            tracing::info!("Received finish status {}", response.status());

            if response.status() == hyper::StatusCode::ACCEPTED {
                tracing::info!("No pending rollup request, trying again");
            } else {
                let body = hyper::body::to_bytes(response).await?;
                let utf = std::str::from_utf8(&body)?;
                let req = json::parse(utf)?;

                let request_type = req["request_type"]
                    .as_str()
                    .ok_or("request_type is not a string")?;
                status = match request_type {
                    "advance_state" => {
                        handle_advance(&client, &server_addr[..], req).await?
                    }
                    "inspect_state" => {
                        handle_inspect(&client, &server_addr[..], req).await?
                    }
                    &_ => {
                        tracing::info!("Unknown request type");
                        "reject"
                    }
                };
            }
        }
    }
}
