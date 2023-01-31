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

///! For more information about each type, see the GraphQL API definition in
///! `offchain/graphql-server/schema.graphql`
use serde::{Deserialize, Serialize};

use crate::{rollups_stream::decl_broker_stream, Address, Hash, Payload};

decl_broker_stream!(RollupsOutputsStream, RollupsOutput, "rollups-outputs");

/// Cartesi  output
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RollupsOutput {
    Voucher(RollupsVoucher),
    Notice(RollupsNotice),
    Report(RollupsReport),
    Proof(RollupsProof),
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsVoucher {
    pub index: u64,
    pub input_index: u64,
    pub destination: Address,
    pub payload: Payload,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsNotice {
    pub index: u64,
    pub input_index: u64,
    pub payload: Payload,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsReport {
    pub index: u64,
    pub input_index: u64,
    pub payload: Payload,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum RollupsOutputEnum {
    #[default]
    Voucher,
    Notice,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsProof {
    pub input_index: u64,
    pub output_index: u64,
    pub output_enum: RollupsOutputEnum,
    pub validity: RollupsOutputValidityProof,
    pub context: Payload,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsOutputValidityProof {
    pub input_index: u64,
    pub output_index: u64,
    pub output_hashes_root_hash: Hash,
    pub vouchers_epoch_root_hash: Hash,
    pub notices_epoch_root_hash: Hash,
    pub machine_state_hash: Hash,
    pub keccak_in_hashes_siblings: Vec<Hash>,
    pub output_hashes_in_epoch_siblings: Vec<Hash>,
}
