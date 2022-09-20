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

use serde::{Deserialize, Serialize};

use crate::broker::BrokerStream;
use crate::serializer_util::{base64_array, base64_vec};
use crate::ADDRESS_SIZE;

#[derive(Debug)]
pub struct RollupsInputsStream {
    key: String,
}

impl RollupsInputsStream {
    pub fn new(chain_id: u64, dapp_id: &[u8; 20]) -> Self {
        Self {
            key: format!(
                "chain-{}:dapp-{}:rollups-inputs",
                chain_id,
                hex::encode(dapp_id)
            ),
        }
    }
}

impl BrokerStream for RollupsInputsStream {
    type Payload = RollupsInput;

    fn key(&self) -> &str {
        &self.key
    }
}

/// Cartesi Rollups event
#[derive(Debug, Serialize, Deserialize)]
pub struct RollupsInput {
    /// Id of the parent of the event
    /// This field must be supplied by the producer of the event.
    /// Notice that the parent might not be the latest event in the stream;
    /// this happens during a reorg.
    /// The parent of the first event should be INITIAL_ID.
    pub parent_id: String,

    /// Epoch index
    pub epoch_index: u64,

    /// Data that depends on the kind of event
    pub data: RollupsData,
}

/// Rollups data enumeration
#[derive(Debug, Serialize, Deserialize)]
pub enum RollupsData {
    /// Input that advances the Cartesi Rollups epoch
    AdvanceStateInput {
        /// Information sent via the input metadata memory range
        input_metadata: InputMetadata,

        /// Payload of the input
        #[serde(with = "base64_vec")]
        input_payload: Vec<u8>,
    },

    /// End of an Cartesi Rollups epoch
    FinishEpoch {},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputMetadata {
    /// Address of the message sender
    #[serde(with = "base64_array")]
    pub msg_sender: [u8; ADDRESS_SIZE],

    /// Block number when the input was posted
    pub block_number: u64,

    /// Timestamp of the block
    pub timestamp: u64,

    /// Epoch index
    pub epoch_index: u64,

    /// Input index in epoch
    pub input_index: u64,
}
