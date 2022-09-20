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

use anyhow::{Context, Result};
use backoff::ExponentialBackoffBuilder;
use rollups_events::rollups_inputs::RollupsData;

use broker::{BrokerFacade, INITIAL_ID};
use config::{Config, ProxyConfig};
use server_manager::ServerManagerFacade;

mod broker;
pub mod config;
mod http_health;
mod server_manager;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: Config) -> Result<()> {
    tracing::info!(?config, "starting proxy");

    let health_handle = tokio::spawn(async move {
        http_health::start_health_check(config.health_check_config).await
    });

    let proxy_handle =
        tokio::spawn(async move { start_proxy(config.proxy_config).await });

    tokio::select! {
        ret = health_handle => {
            ret.context("health-check spawn error")?
                .context("health-check stopped")
        }
        ret = proxy_handle => {
            ret.context("proxy spawn error")?
                .context("proxy stopped")
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
async fn start_proxy(config: ProxyConfig) -> Result<()> {
    let backoff = ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(config.backoff_max_elapsed_duration))
        .build();

    let mut server_manager =
        ServerManagerFacade::new(config.server_manager_config, backoff.clone())
            .await
            .context("failed to connect to the server-manager")?;
    tracing::trace!("connected to the server-manager");

    let mut broker = BrokerFacade::new(config.broker_config, backoff)
        .await
        .context("failed to connect to the broker")?;
    tracing::trace!("connected the broker");

    server_manager
        .start_session()
        .await
        .context("failed to start server-manager session")?;
    tracing::trace!("started server-manager session");

    tracing::info!("starting main loop");
    let mut last_id = INITIAL_ID.to_owned();
    loop {
        let event = broker
            .consume_input(&last_id)
            .await
            .context("failed to consume input from broker")?;
        tracing::info!(?event, "consumed input event");

        tracing::trace!("checking whether parent id match");
        if event.payload.parent_id != last_id {
            return Err(anyhow::Error::msg(
                "broker is inconsistent state; parent id doesn't match",
            ));
        }

        match event.payload.data {
            RollupsData::AdvanceStateInput {
                input_metadata,
                input_payload,
                ..
            } => {
                server_manager
                    .advance_state(input_metadata, input_payload)
                    .await
                    .context(
                        "failed to send advance-state input to server-manager",
                    )?;
                tracing::info!("sent advance-state input to server-manager");
            }
            RollupsData::FinishEpoch { .. } => {
                server_manager
                    .finish_epoch(event.payload.epoch_index)
                    .await
                    .context("failed to finish epoch in server-manager")?;
                tracing::info!("finished epoch in server-manager");

                let claim_produced = broker
                    .was_claim_produced(event.payload.epoch_index)
                    .await
                    .context("failed to get whether claim was produced")?;
                tracing::trace!(
                    claim_produced,
                    "got whether claim was produced"
                );

                if !claim_produced {
                    let claim = server_manager
                        .get_epoch_claim(event.payload.epoch_index)
                        .await
                        .context(
                            "failed to get epoch claim from server-manager",
                        )?;
                    tracing::trace!(
                        claim = hex::encode(claim),
                        "got epoch claim"
                    );

                    broker
                        .produce_rollups_claim(event.payload.epoch_index, claim)
                        .await
                        .context("failed to produce claim in broker")?;
                    tracing::info!("produced epoch claim");
                }
            }
        }
        last_id = event.id;
    }
}
