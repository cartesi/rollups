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

pub mod rollups_broker;

use rollups_events::RollupsClaim;
use types::foldables::input_box::Input;

use async_trait::async_trait;

use self::rollups_broker::BrokerFacadeError;

#[derive(Debug, Default)]
pub struct RollupStatus {
    pub inputs_sent_count: u64,
    pub last_event_is_finish_epoch: bool,
}

#[async_trait]
pub trait BrokerStatus: std::fmt::Debug {
    async fn status(&self) -> Result<RollupStatus, BrokerFacadeError>;
}

#[async_trait]
pub trait BrokerSend: std::fmt::Debug {
    async fn enqueue_input(
        &self,
        input_index: u64,
        input: &Input,
    ) -> Result<(), BrokerFacadeError>;
    async fn finish_epoch(
        &self,
        inputs_sent_count: u64,
    ) -> Result<(), BrokerFacadeError>;
}

#[async_trait]
pub trait BrokerReceive: std::fmt::Debug {
    async fn next_claim(
        &self,
    ) -> Result<Option<RollupsClaim>, BrokerFacadeError>;
}
