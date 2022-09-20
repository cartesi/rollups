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
use im::Vector;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tokio::sync::Mutex;

use rollups_events::broker::{Broker, BrokerError, INITIAL_ID};
use rollups_events::rollups_claims::RollupsClaimsStream;
use rollups_events::rollups_inputs::{
    InputMetadata, RollupsData, RollupsInput, RollupsInputsStream,
};
use state_fold_types::ethabi::ethereum_types::{H256, U256};
use types::input::Input;

use super::{config::BrokerConfig, EpochStatus, MachineInterface};

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
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_last_input_id(&self) -> Result<String> {
        tracing::trace!("getting id of last produced event");

        let mut broker = self.broker.lock().await;
        let response = broker
            .peek_latest(&self.inputs_stream)
            .await
            .context(PeekInputSnafu)?;

        tracing::trace!(?response, "got response");

        let last_id = match response {
            Some(event) => event.id,
            None => INITIAL_ID.to_owned(),
        };

        tracing::trace!(last_id, "returning last id");

        Ok(last_id)
    }
}

#[async_trait]
impl MachineInterface for BrokerFacade {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_current_epoch_status(&self) -> Result<EpochStatus> {
        tracing::trace!("getting current epoch status");

        tracing::trace!("peeking latest event in inputs stream");
        let mut broker = self.broker.lock().await;
        let response = broker
            .peek_latest(&self.inputs_stream)
            .await
            .context(PeekInputSnafu)?;

        tracing::trace!(?response, "got response");

        let status = match response {
            Some(event) => {
                match event.payload.data {
                    RollupsData::AdvanceStateInput {
                        input_metadata, ..
                    } => {
                        let epoch_number = event.payload.epoch_index.into();
                        let processed_input_count =
                            input_metadata.input_index as usize + 1;
                        EpochStatus {
                            epoch_number,
                            processed_input_count,
                            pending_input_count: 0,
                            is_active: true,
                        }
                    }
                    RollupsData::FinishEpoch { .. } => {
                        // If the epoch finished return the next epoch as active
                        let epoch_number =
                            U256::from(event.payload.epoch_index + 1);
                        EpochStatus {
                            epoch_number,
                            processed_input_count: 0,
                            pending_input_count: 0,
                            is_active: true,
                        }
                    }
                }
            }
            None => EpochStatus {
                epoch_number: U256::from(0),
                processed_input_count: 0,
                pending_input_count: 0,
                is_active: true,
            },
        };

        tracing::trace!(?status, "returning epoch status");
        Ok(status)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn enqueue_inputs(
        &self,
        epoch_number: U256,
        first_input_index: U256,
        inputs: Vector<Arc<Input>>,
    ) -> Result<()> {
        tracing::trace!(
            ?epoch_number,
            ?first_input_index,
            ?inputs,
            "enqueueing inputs"
        );

        let mut last_id = self.get_last_input_id().await?;
        let mut input_index = first_input_index.as_u64();

        for input in inputs {
            let mut broker = self.broker.lock().await;

            let input_metadata = InputMetadata {
                msg_sender: input.sender.to_fixed_bytes(),
                block_number: input.block_number.as_u64(),
                timestamp: input.timestamp.as_u64(),
                epoch_index: epoch_number.as_u64(),
                input_index,
            };
            let data = RollupsData::AdvanceStateInput {
                input_metadata,
                input_payload: (*input.payload).clone(),
            };
            let event = RollupsInput {
                parent_id: last_id.clone(),
                epoch_index: epoch_number.as_u64(),
                data,
            };

            tracing::trace!(?event, "producing input event");

            let id = broker
                .produce(&self.inputs_stream, event)
                .await
                .context(ProduceInputSnafu)?;

            tracing::trace!(id, "produced event with id");

            last_id = id;
            input_index += 1;
        }

        tracing::trace!("finished producing events");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn finish_epoch(
        &self,
        epoch_number: U256,
        _input_count: U256,
    ) -> Result<()> {
        tracing::trace!(?epoch_number, "finishing epoch");

        let last_id = self.get_last_input_id().await?;

        let mut broker = self.broker.lock().await;

        let event = RollupsInput {
            parent_id: last_id.clone(),
            epoch_index: epoch_number.as_u64(),
            data: RollupsData::FinishEpoch {},
        };

        tracing::trace!(?event, "producing finish epoch event");

        let id = broker
            .produce(&self.inputs_stream, event)
            .await
            .context(ProduceFinishSnafu)?;

        tracing::trace!(id, "produce event with id");

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_epoch_claim(&self, epoch_number: U256) -> Result<H256> {
        tracing::trace!(?epoch_number, "getting epoch claim");

        let mut last_id = INITIAL_ID.to_owned();

        // This loop goes through the stream searching for the expected epoch claim.
        // If the expected epoch is not found, the loop will eventually fail with a timeout.
        loop {
            tracing::trace!(last_id, "consuming from claims stream");

            let mut broker = self.broker.lock().await;
            let event = broker
                .consume_block(&self.claims_stream, &last_id)
                .await
                .context(ConsumeClaimSnafu)?;

            tracing::trace!(?event, "consumed event");

            if event.payload.epoch_index == epoch_number.as_u64() {
                tracing::trace!("found claim");
                return Ok(event.payload.claim.into());
            }
            last_id = event.id;
        }
    }
}
