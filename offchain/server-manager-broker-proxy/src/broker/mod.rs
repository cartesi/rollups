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
use rollups_events::broker::{Broker, BrokerError, Event, INITIAL_ID};
use rollups_events::rollups_claims::{RollupsClaim, RollupsClaimsStream};
use rollups_events::rollups_inputs::{
    RollupsData, RollupsInput, RollupsInputsStream,
};
use snafu::{ResultExt, Snafu};

use config::BrokerConfig;

pub mod config;

#[derive(Debug, Snafu)]
pub enum BrokerFacadeError {
    #[snafu(display("broker internal error"))]
    BrokerInternalError { source: BrokerError },

    #[snafu(display("failed to consume input event"))]
    ConsumeError { source: BrokerError },

    #[snafu(display(
        "failed to find finish epoch input event epoch={}",
        epoch
    ))]
    FindFinishEpochInputError { epoch: u64 },

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

    /// Search the input event stream for the finish epoch event of the previous epoch
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn find_previous_finish_epoch(
        &mut self,
        mut epoch: u64,
    ) -> Result<String> {
        tracing::trace!(epoch, "getting previous finish epoch");

        if epoch == 0 {
            tracing::trace!("returning initial id for epoch 0");
            return Ok(INITIAL_ID.to_owned());
        } else {
            // This won't underflow because we know the epoch is not 0
            epoch = epoch - 1;
        }

        tracing::trace!(epoch, "searching for finish epoch input event");
        let mut last_id = INITIAL_ID.to_owned();
        loop {
            let event = self
                .client
                .consume_nonblock(&self.inputs_stream, &last_id)
                .await
                .context(ConsumeSnafu)?
                .ok_or(BrokerFacadeError::FindFinishEpochInputError {
                    epoch,
                })?;
            if matches!(
                event.payload,
                RollupsInput {
                    data: RollupsData::FinishEpoch {},
                    epoch_index,
                    ..
                } if epoch_index == epoch
            ) {
                tracing::trace!(event_id = last_id, "returning event id");
                return Ok(event.id);
            }
            last_id = event.id;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rollups_events::rollups_inputs::InputMetadata;
    use rollups_events::HASH_SIZE;
    use testcontainers::*;

    struct TestState<'d> {
        _node: Container<'d, images::redis::Redis>,
        broker: Broker,
        facade: BrokerFacade,
    }

    impl TestState<'_> {
        async fn setup(docker: &clients::Cli) -> TestState<'_> {
            let node = docker.run(images::redis::Redis::default());
            let port = node.get_host_port_ipv4(6379);
            let redis_endpoint = format!("redis://127.0.0.1:{}", port);
            let backoff = ExponentialBackoff::default();
            let broker = Broker::new(&redis_endpoint, backoff.clone(), 10)
                .await
                .expect("failed to connect to broker");
            let config = BrokerConfig {
                redis_endpoint,
                chain_id: 0,
                dapp_contract_address: [0xfa; 20],
                consume_timeout: 10,
            };
            let facade = BrokerFacade::new(config, backoff)
                .await
                .expect("failed to create broker facade");
            TestState {
                _node: node,
                broker,
                facade,
            }
        }

        async fn get_latest_input_event(
            &mut self,
        ) -> Option<Event<RollupsInput>> {
            self.broker
                .peek_latest(&self.facade.inputs_stream)
                .await
                .unwrap()
        }

        /// Produces the input events given the data
        /// Returns the produced events ids
        async fn produce_input_events(
            &mut self,
            datum: Vec<RollupsData>,
        ) -> Vec<String> {
            let mut ids = vec![];
            for data in datum {
                let last_event = self.get_latest_input_event().await;
                let (parent_id, epoch_index, inputs_sent_count) =
                    match last_event {
                        Some(event) => {
                            let epoch_index = match event.payload.data {
                                RollupsData::AdvanceStateInput { .. } => {
                                    event.payload.epoch_index
                                }
                                RollupsData::FinishEpoch {} => {
                                    event.payload.epoch_index + 1
                                }
                            };
                            (
                                event.id,
                                epoch_index,
                                event.payload.inputs_sent_count + 1,
                            )
                        }
                        None => (INITIAL_ID.to_owned(), 0, 0),
                    };
                let id = self
                    .broker
                    .produce(
                        &self.facade.inputs_stream,
                        RollupsInput {
                            parent_id,
                            epoch_index,
                            inputs_sent_count,
                            data,
                        },
                    )
                    .await
                    .unwrap();
                ids.push(id.clone());
            }
            ids
        }

        /// Produce the claims given the hashes
        async fn produce_claims(&mut self, claims: Vec<[u8; HASH_SIZE]>) {
            for claim in claims {
                let last_claim = self
                    .broker
                    .peek_latest(&self.facade.claims_stream)
                    .await
                    .unwrap();
                let epoch_index = match last_claim {
                    Some(event) => event.payload.epoch_index + 1,
                    None => 0,
                };
                self.broker
                    .produce(
                        &self.facade.claims_stream,
                        RollupsClaim { epoch_index, claim },
                    )
                    .await
                    .unwrap();
            }
        }

        /// Obtain all produced claims
        async fn consume_claims(&mut self) -> Vec<RollupsClaim> {
            let mut claims = vec![];
            let mut last_id = INITIAL_ID.to_owned();
            while let Some(event) = self
                .broker
                .consume_nonblock(&self.facade.claims_stream, &last_id)
                .await
                .unwrap()
            {
                claims.push(event.payload);
                last_id = event.id;
            }
            claims
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_it_finds_previous_finish_of_first_epoch() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        let id = state.facade.find_previous_finish_epoch(0).await.unwrap();
        assert_eq!(id, INITIAL_ID);
    }

    #[test_log::test(tokio::test)]
    async fn test_it_finds_previous_finish_of_nth_epoch() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        let ids = state
            .produce_input_events(vec![
                RollupsData::FinishEpoch {},
                RollupsData::FinishEpoch {},
            ])
            .await;
        assert_eq!(
            state.facade.find_previous_finish_epoch(1).await.unwrap(),
            ids[0]
        );
        assert_eq!(
            state.facade.find_previous_finish_epoch(2).await.unwrap(),
            ids[1]
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_find_previous_epoch_when_it_is_missing() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        state
            .produce_input_events(vec![
                RollupsData::FinishEpoch {},
                RollupsData::FinishEpoch {},
            ])
            .await;
        assert!(matches!(
            state
                .facade
                .find_previous_finish_epoch(3)
                .await
                .unwrap_err(),
            BrokerFacadeError::FindFinishEpochInputError { epoch: 2 }
        ));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_consumes_inputs() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        let inputs_data = vec![
            RollupsData::AdvanceStateInput {
                input_metadata: InputMetadata {
                    block_number: 0,
                    epoch_index: 0,
                    input_index: 0,
                    msg_sender: [0xfa; 20],
                    timestamp: 0,
                },
                input_payload: vec![0, 0],
            },
            RollupsData::FinishEpoch {},
            RollupsData::AdvanceStateInput {
                input_metadata: InputMetadata {
                    block_number: 0,
                    epoch_index: 1,
                    input_index: 1,
                    msg_sender: [0xfa; 20],
                    timestamp: 0,
                },
                input_payload: vec![1, 1],
            },
        ];
        let ids = state.produce_input_events(inputs_data.clone()).await;
        assert_eq!(
            state.facade.consume_input(INITIAL_ID).await.unwrap(),
            Event {
                id: ids[0].clone(),
                payload: RollupsInput {
                    parent_id: INITIAL_ID.to_owned(),
                    epoch_index: 0,
                    inputs_sent_count: 0,
                    data: inputs_data[0].clone(),
                },
            }
        );
        assert_eq!(
            state.facade.consume_input(&ids[0]).await.unwrap(),
            Event {
                id: ids[1].clone(),
                payload: RollupsInput {
                    parent_id: ids[0].clone(),
                    epoch_index: 0,
                    inputs_sent_count: 1,
                    data: inputs_data[1].clone(),
                },
            }
        );
        assert_eq!(
            state.facade.consume_input(&ids[1]).await.unwrap(),
            Event {
                id: ids[2].clone(),
                payload: RollupsInput {
                    parent_id: ids[1].clone(),
                    epoch_index: 1,
                    inputs_sent_count: 2,
                    data: inputs_data[2].clone(),
                },
            }
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_it_checks_claim_was_produced() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        state
            .produce_claims(vec![
                [0xa0; HASH_SIZE],
                [0xa1; HASH_SIZE],
                [0xa2; HASH_SIZE],
            ])
            .await;
        assert!(state.facade.was_claim_produced(0).await.unwrap());
        assert!(state.facade.was_claim_produced(1).await.unwrap());
        assert!(state.facade.was_claim_produced(2).await.unwrap());
    }

    #[test_log::test(tokio::test)]
    async fn test_it_checks_claim_was_not_produced() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        state
            .produce_claims(vec![
                [0xa0; HASH_SIZE],
                [0xa1; HASH_SIZE],
                [0xa2; HASH_SIZE],
            ])
            .await;
        assert!(!state.facade.was_claim_produced(3).await.unwrap());
        assert!(!state.facade.was_claim_produced(4).await.unwrap());
    }

    #[test_log::test(tokio::test)]
    async fn test_it_produces_claims() {
        let docker = clients::Cli::default();
        let mut state = TestState::setup(&docker).await;
        state
            .facade
            .produce_rollups_claim(0, [0xa0; HASH_SIZE])
            .await
            .unwrap();
        state
            .facade
            .produce_rollups_claim(1, [0xa1; HASH_SIZE])
            .await
            .unwrap();
        assert_eq!(
            state.consume_claims().await,
            vec![
                RollupsClaim {
                    epoch_index: 0,
                    claim: [0xa0; HASH_SIZE],
                },
                RollupsClaim {
                    epoch_index: 1,
                    claim: [0xa1; HASH_SIZE],
                },
            ]
        );
    }
}
