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

pub use rollups_http_client::rollup::*;

use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashMap;

use super::config;

#[derive(Debug, PartialEq)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "request_type")]
pub enum RollupHttpRequest {
    #[serde(rename = "advance_state")]
    Advance { data: AdvanceRequest },
    #[serde(rename = "inspect_state")]
    Inspect { data: InspectRequest },
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct InspectStateResponse {
    pub reports: Vec<Report>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct IndexResponse {
    pub index: usize,
}

pub fn convert_binary_to_hex(payload: &Vec<u8>) -> String {
    String::from("0x") + &hex::encode(payload)
}

pub fn create_address() -> String {
    convert_binary_to_hex(&super::create_address())
}

pub fn create_payload() -> String {
    convert_binary_to_hex(&super::create_payload())
}

pub async fn finish(status: String) -> Result<RollupHttpRequest, HttpError> {
    let url = format!("{}/finish", config::get_http_rollup_server_address());
    let mut request = HashMap::new();
    request.insert("status", status);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await.unwrap();
    handle_json_response(response).await
}

pub async fn insert_voucher(
    destination: String,
    payload: String,
) -> Result<IndexResponse, HttpError> {
    let url = format!("{}/voucher", config::get_http_rollup_server_address());
    let mut request = HashMap::new();
    request.insert("destination", destination);
    request.insert("payload", payload);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await.unwrap();
    handle_json_response(response).await
}

pub async fn insert_notice(
    payload: String,
) -> Result<IndexResponse, HttpError> {
    let url = format!("{}/notice", config::get_http_rollup_server_address());
    let mut request = HashMap::new();
    request.insert("payload", payload);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await.unwrap();
    handle_json_response(response).await
}

pub async fn insert_report(payload: String) -> Result<(), HttpError> {
    let url = format!("{}/report", config::get_http_rollup_server_address());
    let mut request = HashMap::new();
    request.insert("payload", payload);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await.unwrap();
    handle_response(response).await.map(|_| ())
}

pub async fn notify_exception(payload: String) -> Result<(), HttpError> {
    let url = format!("{}/exception", config::get_http_rollup_server_address());
    let mut request = HashMap::new();
    request.insert("payload", payload);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await.unwrap();
    handle_response(response).await.map(|_| ())
}

async fn handle_response(response: Response) -> Result<Response, HttpError> {
    if response.status() == reqwest::StatusCode::OK {
        Ok(response)
    } else {
        Err(HttpError {
            status: response.status().as_u16(),
            message: response.text().await.unwrap(),
        })
    }
}

async fn handle_json_response<T: DeserializeOwned>(
    response: Response,
) -> Result<T, HttpError> {
    match handle_response(response).await {
        Ok(response) => Ok(response.json::<T>().await.unwrap()),
        Err(e) => Err(e),
    }
}
