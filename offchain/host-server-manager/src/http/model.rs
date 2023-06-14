// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::conversions::{self, DecodeError};
use crate::model::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpAdvanceMetadata {
    pub msg_sender: String,
    pub epoch_index: u64,
    pub input_index: u64,
    pub block_number: u64,
    pub timestamp: u64,
}

impl From<AdvanceMetadata> for HttpAdvanceMetadata {
    fn from(metadata: AdvanceMetadata) -> HttpAdvanceMetadata {
        HttpAdvanceMetadata {
            msg_sender: conversions::encode_ethereum_binary(&metadata.msg_sender),
            epoch_index: metadata.epoch_index,
            input_index: metadata.input_index,
            block_number: metadata.block_number,
            timestamp: metadata.timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpAdvanceStateRequest {
    pub metadata: HttpAdvanceMetadata,
    pub payload: String,
}

impl From<AdvanceStateRequest> for HttpAdvanceStateRequest {
    fn from(request: AdvanceStateRequest) -> HttpAdvanceStateRequest {
        HttpAdvanceStateRequest {
            metadata: request.metadata.into(),
            payload: conversions::encode_ethereum_binary(&request.payload),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpInspectStateRequest {
    pub payload: String,
}

impl From<InspectStateRequest> for HttpInspectStateRequest {
    fn from(request: InspectStateRequest) -> HttpInspectStateRequest {
        HttpInspectStateRequest {
            payload: conversions::encode_ethereum_binary(&request.payload),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request_type")]
pub enum HttpRollupRequest {
    #[serde(rename = "advance_state")]
    AdvanceState { data: HttpAdvanceStateRequest },
    #[serde(rename = "inspect_state")]
    InspectState { data: HttpInspectStateRequest },
}

impl From<RollupRequest> for HttpRollupRequest {
    fn from(request: RollupRequest) -> HttpRollupRequest {
        match request {
            RollupRequest::AdvanceState(request) => HttpRollupRequest::AdvanceState {
                data: request.into(),
            },
            RollupRequest::InspectState(request) => HttpRollupRequest::InspectState {
                data: request.into(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpVoucher {
    pub destination: String,
    pub payload: String,
}

impl TryFrom<HttpVoucher> for Voucher {
    type Error = VoucherDecodeError;
    fn try_from(voucher: HttpVoucher) -> Result<Voucher, VoucherDecodeError> {
        Ok(Voucher::new(
            conversions::decode_ethereum_binary(&voucher.destination)?.try_into()?,
            conversions::decode_ethereum_binary(&voucher.payload)?,
        ))
    }
}

#[derive(Debug, Snafu)]
pub enum VoucherDecodeError {
    #[snafu(display("Invalid Ethereum address size (got {} bytes, expected 20 bytes)", got))]
    InvalidAddressSize { got: usize },
    #[snafu(display("{}", e))]
    HexDecodeError { e: DecodeError },
}

impl From<DecodeError> for VoucherDecodeError {
    fn from(e: DecodeError) -> VoucherDecodeError {
        VoucherDecodeError::HexDecodeError { e }
    }
}

impl From<Vec<u8>> for VoucherDecodeError {
    fn from(bytes: Vec<u8>) -> VoucherDecodeError {
        VoucherDecodeError::InvalidAddressSize { got: bytes.len() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpNotice {
    pub payload: String,
}

impl TryFrom<HttpNotice> for Notice {
    type Error = DecodeError;
    fn try_from(notice: HttpNotice) -> Result<Notice, DecodeError> {
        Ok(Notice::new(conversions::decode_ethereum_binary(
            &notice.payload,
        )?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpReport {
    pub payload: String,
}

impl From<Report> for HttpReport {
    fn from(report: Report) -> HttpReport {
        HttpReport {
            payload: conversions::encode_ethereum_binary(&report.payload),
        }
    }
}

impl TryFrom<HttpReport> for Report {
    type Error = DecodeError;
    fn try_from(report: HttpReport) -> Result<Report, DecodeError> {
        Ok(Report {
            payload: conversions::decode_ethereum_binary(&report.payload)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRollupException {
    pub payload: String,
}

impl TryFrom<HttpRollupException> for RollupException {
    type Error = DecodeError;
    fn try_from(report: HttpRollupException) -> Result<RollupException, DecodeError> {
        Ok(RollupException {
            payload: conversions::decode_ethereum_binary(&report.payload)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpFinishRequest {
    pub status: String,
}

impl TryFrom<HttpFinishRequest> for FinishStatus {
    type Error = DecodeStatusError;
    fn try_from(request: HttpFinishRequest) -> Result<FinishStatus, DecodeStatusError> {
        match request.status.as_str() {
            "accept" => Ok(FinishStatus::Accept),
            "reject" => Ok(FinishStatus::Reject),
            _ => Err(DecodeStatusError {}),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpIndexResponse {
    pub index: u64,
}

#[derive(Debug, Snafu)]
#[snafu(display("status must be 'accept' or 'reject'"))]
pub struct DecodeStatusError {}
