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

use serde::{Deserialize, Serialize};

use crate::{rollups_stream::decl_broker_stream, Hash};

decl_broker_stream!(RollupsClaimsStream, RollupsClaim, "rollups-claims");

/// Event generated when the Cartesi Rollups epoch finishes
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RollupsClaim {
    /// Epoch index
    pub epoch_index: u64,

    /// Hash of the Epoch
    pub epoch_hash: Hash,

    /// Index of the first input of the Epoch
    pub first_index: u128,

    /// Index of the last input of the Epoch
    pub last_index: u128,
}
