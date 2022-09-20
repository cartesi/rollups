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

use backoff::ExponentialBackoff;
use rollups_events::broker::BrokerError;
pub use rollups_events::broker::INITIAL_ID;
use rollups_events::broker::{Broker, Event};
use rollups_events::rollups_claims::{RollupsClaim, RollupsClaimsStream};
use rollups_events::rollups_inputs::{RollupsInput, RollupsInputsStream};
use snafu::{ResultExt, Snafu};

use config::BrokerConfig;

pub mod config;

#[derive(Debug, Snafu)]
pub enum BrokerFacadeError {
    #[snafu(display("broker internal error"))]
    BrokerInternalError { source: BrokerError },

    #[snafu(display("processed event not found in broker"))]
    ProcessedEventNotFound {},
}

pub type Result<T> = std::result::Result<T, BrokerFacadeError>;

pub struct BrokerFacade {
    client: Broker,
    inputs_stream: RollupsInputsStream,
    claims_stream: RollupsClaimsStream,
}

impl BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn new(
        config: BrokerConfig,
        backoff: ExponentialBackoff,
    ) -> Result<Self> {
        tracing::trace!(?config, "connecting to broker");

        let client = Broker::new(
            &config.redis_endpoint,
            backoff,
            config.consume_timeout,
        )
        .await
        .context(BrokerInternalSnafu)?;

        let inputs_stream = RollupsInputsStream::new(
            config.chain_id,
            &config.dapp_contract_address,
        );
        let claims_stream = RollupsClaimsStream::new(
            config.chain_id,
            &config.dapp_contract_address,
        );

        Ok(Self {
            client,
            inputs_stream,
            claims_stream,
        })
    }

    /// Consume rollups input event
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn consume_input(
        &mut self,
        last_id: &str,
    ) -> Result<Event<RollupsInput>> {
        tracing::trace!(last_id, "consuming rollups input event");

        loop {
            let result = self
                .client
                .consume_block(&self.inputs_stream, &last_id)
                .await;
            if matches!(result, Err(BrokerError::ConsumeTimeout)) {
                tracing::trace!("consume timed out, retrying");
            } else {
                return result.context(BrokerInternalSnafu);
            }
        }
    }

    /// Obtain the epoch number of the last generated claim
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn was_claim_produced(
        &mut self,
        epoch_index: u64,
    ) -> Result<bool> {
        tracing::trace!(epoch_index, "peeking last generated claim");

        let result = self
            .client
            .peek_latest(&self.claims_stream)
            .await
            .context(BrokerInternalSnafu)?;

        Ok(match result {
            Some(event) => {
                tracing::trace!(?event, "got last claim produced");
                epoch_index <= event.payload.epoch_index
            }
            None => {
                tracing::trace!("no claims in the stream");
                false
            }
        })
    }

    /// Obtain the epoch hashes from the server-manager and generate the rollups claim
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn produce_rollups_claim(
        &mut self,
        epoch_index: u64,
        claim: [u8; 32],
    ) -> Result<()> {
        tracing::trace!(
            epoch_index,
            claim = hex::encode(claim),
            "producing rollups claim"
        );

        let rollups_claim = RollupsClaim { epoch_index, claim };
        self.client
            .produce(&self.claims_stream, rollups_claim)
            .await
            .context(BrokerInternalSnafu)?;

        Ok(())
    }
}
