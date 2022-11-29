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
use crate::serializer_util::base64_array;
use crate::HASH_SIZE;

#[derive(Debug)]
pub struct RollupsClaimsStream {
    key: String,
}

impl RollupsClaimsStream {
    pub fn new(chain_id: u64, dapp_id: &[u8; 20]) -> Self {
        Self {
            key: format!(
                "chain-{}:dapp-{}:rollups-claims",
                chain_id,
                hex::encode(dapp_id)
            ),
        }
    }
}

impl BrokerStream for RollupsClaimsStream {
    type Payload = RollupsClaim;

    fn key(&self) -> &str {
        &self.key
    }
}

/// Event generated when the Cartesi Rollups epoch finishes
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsClaim {
    /// Epoch index
    pub epoch_index: u64,

    /// Hash of the Epoch
    #[serde(with = "base64_array")]
    pub claim: [u8; HASH_SIZE],
}
