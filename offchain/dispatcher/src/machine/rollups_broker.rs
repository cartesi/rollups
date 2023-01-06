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

use anyhow::Result;
use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use snafu::{ResultExt, Snafu};
use tokio::sync::{self, Mutex};

use rollups_events::{
    broker::{Broker, BrokerError, INITIAL_ID},
    rollups_claims::{RollupsClaim, RollupsClaimsStream},
    rollups_inputs::{
        InputMetadata, RollupsData, RollupsInput, RollupsInputsStream,
    },
};
use types::foldables::input_box::Input;

use super::{
    config::BrokerConfig, BrokerReceive, BrokerSend, BrokerStatus, RollupStatus,
};

#[derive(Debug, Snafu)]
pub enum BrokerFacadeError {
    #[snafu(display("error connecting to the broker"))]
    BrokerConnectionError { source: BrokerError },

    #[snafu(display("error peeking at the end of the stream"))]
    PeekInputError { source: BrokerError },

    #[snafu(display("error producing input event"))]
    ProduceInputError { source: BrokerError },

    #[snafu(display("error producing finish-epoch event"))]
    ProduceFinishError { source: BrokerError },

    #[snafu(display("error consuming claim event"))]
    ConsumeClaimError { source: BrokerError },
}

#[derive(Debug)]
pub struct BrokerFacade {
    broker: Mutex<Broker>,
    inputs_stream: RollupsInputsStream,
    claims_stream: RollupsClaimsStream,
    last_claim_id: Mutex<String>,
}

struct BrokerStreamStatus {
    id: String,
    epoch_number: u64,
    status: RollupStatus,
}

impl BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn new(config: BrokerConfig) -> Result<Self> {
        tracing::trace!(?config, "connection to the broker");

        let backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(config.backoff_max_elapsed_duration))
            .build();
        let broker = Mutex::new(
            Broker::new(
                &config.redis_endpoint,
                backoff,
                config.claims_consume_timeout,
            )
            .await
            .context(BrokerConnectionSnafu)?,
        );

        tracing::trace!("connected to the broker successfully");

        let inputs_stream = RollupsInputsStream::new(
            config.chain_id,
            &config.dapp_contract_address,
        );

        let claims_stream = RollupsClaimsStream::new(
            config.chain_id,
            &config.dapp_contract_address,
        );

        Ok(Self {
            broker,
            inputs_stream,
            claims_stream,
            last_claim_id: Mutex::new(INITIAL_ID.to_owned()),
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn broker_status(
        &self,
        broker: &mut sync::MutexGuard<'_, Broker>,
    ) -> Result<BrokerStreamStatus> {
        let event = self.peek(broker).await?;
        Ok(event.into())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn peek(
        &self,
        broker: &mut sync::MutexGuard<'_, Broker>,
    ) -> Result<Option<rollups_events::broker::Event<RollupsInput>>> {
        tracing::trace!("peeking last produced event");
        let response = broker
            .peek_latest(&self.inputs_stream)
            .await
            .context(PeekInputSnafu)?;
        tracing::trace!(?response, "got response");

        Ok(response)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn claim(
        &self,
        id: &String,
    ) -> Result<Option<rollups_events::broker::Event<RollupsClaim>>> {
        let mut broker = self.broker.lock().await;
        let event = broker
            .consume_nonblock(&self.claims_stream, id)
            .await
            .context(ConsumeClaimSnafu)?;

        tracing::trace!(?event, "consumed event");

        Ok(event)
    }
}

#[async_trait]
impl BrokerStatus for BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn status(&self) -> Result<RollupStatus> {
        tracing::trace!("querying broker status");
        let mut broker = self.broker.lock().await;
        let status = self.broker_status(&mut broker).await?.status;
        tracing::trace!(?status, "returning rollup status");
        Ok(status)
    }
}

macro_rules! input_sanity_check {
    ($event:expr, $input_index:expr) => {
        assert_eq!($event.inputs_sent_count, $input_index + 1);
        assert!(matches!(
            $event.data,
            RollupsData::AdvanceStateInput {
                input_metadata: InputMetadata {
                    epoch_index,
                    ..
                },
                ..
            } if epoch_index == 0
        ));
        assert!(matches!(
            $event.data,
            RollupsData::AdvanceStateInput {
                input_metadata: InputMetadata {
                    input_index,
                    ..
                },
                ..
            } if input_index == $input_index
        ));
    };
}

macro_rules! epoch_sanity_check {
    ($event:expr, $inputs_sent_count:expr) => {
        assert_eq!($event.inputs_sent_count, $inputs_sent_count);
        assert!(matches!($event.data, RollupsData::FinishEpoch { .. }));
    };
}

#[async_trait]
impl BrokerSend for BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn enqueue_input(
        &self,
        input_index: u64,
        input: &Input,
    ) -> Result<()> {
        tracing::trace!(?input_index, ?input, "enqueueing input");

        let mut broker = self.broker.lock().await;
        let status = self.broker_status(&mut broker).await?;

        let event = build_next_input(input, &status);
        tracing::trace!(?event, "producing input event");

        input_sanity_check!(event, input_index);

        let id = broker
            .produce(&self.inputs_stream, event)
            .await
            .context(ProduceInputSnafu)?;
        tracing::trace!(id, "produced event with id");

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn finish_epoch(&self, inputs_sent_count: u64) -> Result<()> {
        tracing::trace!(?inputs_sent_count, "finishing epoch");

        let mut broker = self.broker.lock().await;
        let status = self.broker_status(&mut broker).await?;

        let event = build_next_finish_epoch(&status);
        tracing::trace!(?event, "producing finish epoch event");

        epoch_sanity_check!(event, inputs_sent_count);

        let id = broker
            .produce(&self.inputs_stream, event)
            .await
            .context(ProduceFinishSnafu)?;

        tracing::trace!(id, "produce event with id");

        Ok(())
    }
}

#[async_trait]
impl BrokerReceive for BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn next_claim(&self) -> Result<Option<super::RollupClaim>> {
        let mut last_id = self.last_claim_id.lock().await;
        tracing::trace!(?last_id, "getting next epoch claim");

        match self.claim(&last_id).await? {
            Some(event) => {
                *last_id = event.id.clone();
                Ok(Some(event.into()))
            }
            None => Ok(None),
        }
    }
}

impl From<RollupsInput> for RollupStatus {
    fn from(payload: RollupsInput) -> Self {
        let inputs_sent_count = payload.inputs_sent_count;

        match payload.data {
            RollupsData::AdvanceStateInput { .. } => RollupStatus {
                inputs_sent_count,
                last_event_is_finish_epoch: false,
            },

            RollupsData::FinishEpoch { .. } => RollupStatus {
                inputs_sent_count,
                last_event_is_finish_epoch: true,
            },
        }
    }
}

impl From<rollups_events::broker::Event<RollupsInput>> for BrokerStreamStatus {
    fn from(event: rollups_events::broker::Event<RollupsInput>) -> Self {
        let id = event.id;
        let payload = event.payload;
        let epoch_index = payload.epoch_index;

        match payload.data {
            RollupsData::AdvanceStateInput { .. } => Self {
                id,
                epoch_number: epoch_index,
                status: payload.into(),
            },

            RollupsData::FinishEpoch { .. } => Self {
                id,
                epoch_number: epoch_index + 1,
                status: payload.into(),
            },
        }
    }
}

impl From<Option<rollups_events::broker::Event<RollupsInput>>>
    for BrokerStreamStatus
{
    fn from(
        event: Option<rollups_events::broker::Event<RollupsInput>>,
    ) -> Self {
        match event {
            Some(e) => e.into(),

            None => Self {
                id: INITIAL_ID.to_owned(),
                epoch_number: 0,
                status: RollupStatus::default(),
            },
        }
    }
}

fn build_next_input(
    input: &Input,
    status: &BrokerStreamStatus,
) -> RollupsInput {
    let input_metadata = InputMetadata {
        msg_sender: input.sender.to_fixed_bytes(),
        block_number: input.block_added.number.as_u64(),
        timestamp: input.block_added.timestamp.as_u64(),
        epoch_index: 0,
        input_index: status.status.inputs_sent_count,
    };

    let data = RollupsData::AdvanceStateInput {
        input_metadata,
        input_payload: input.payload.clone(),
    };

    RollupsInput {
        parent_id: status.id.clone(),
        epoch_index: status.epoch_number,
        inputs_sent_count: status.status.inputs_sent_count + 1,
        data,
    }
}

fn build_next_finish_epoch(status: &BrokerStreamStatus) -> RollupsInput {
    RollupsInput {
        parent_id: status.id.clone(),
        epoch_index: status.epoch_number,
        inputs_sent_count: status.status.inputs_sent_count,
        data: RollupsData::FinishEpoch {},
    }
}

impl From<rollups_events::broker::Event<RollupsClaim>> for super::RollupClaim {
    fn from(event: rollups_events::broker::Event<RollupsClaim>) -> Self {
        super::RollupClaim {
            hash: event.payload.claim,
            number: event.payload.epoch_index,
        }
    }
}
