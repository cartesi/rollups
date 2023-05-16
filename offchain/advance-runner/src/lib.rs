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

use anyhow::{Context, Result};
use backoff::ExponentialBackoffBuilder;

use broker::BrokerFacade;
use config::{AdvanceRunnerConfig, Config};
use runner::Runner;
use server_manager::ServerManagerFacade;
use snapshot::{
    config::SnapshotConfig, disabled::SnapshotDisabled,
    fs_manager::FSSnapshotManager,
};

mod broker;
pub mod config;
mod runner;
mod server_manager;
mod snapshot;

#[tracing::instrument(level = "trace", skip_all)]
pub async fn run(config: Config) -> Result<()> {
    tracing::info!(?config, "starting advance runner");

    let health_handle = http_health_check::start(config.health_check_config);
    let advance_runner_handle =
        start_advance_runner(config.advance_runner_config);

    tokio::select! {
        ret = health_handle => {
            ret.context("health-check stopped")
        }
        ret = advance_runner_handle => {
            ret.context("advance runner stopped")
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
async fn start_advance_runner(config: AdvanceRunnerConfig) -> Result<()> {
    let backoff = ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(config.backoff_max_elapsed_duration))
        .build();

    let server_manager =
        ServerManagerFacade::new(config.server_manager_config, backoff)
            .await
            .context("failed to connect to the server-manager")?;
    tracing::trace!("connected to the server-manager");

    let broker = BrokerFacade::new(config.broker_config, config.dapp_metadata)
        .await
        .context("failed to connect to the broker")?;
    tracing::trace!("connected the broker");

    match config.snapshot_config {
        SnapshotConfig::FileSystem(fs_manager_config) => {
            let snapshot_manager = FSSnapshotManager::new(fs_manager_config);
            Runner::start(server_manager, broker, snapshot_manager)
                .await
                .context("error in the advance runner main loop")
        }
        SnapshotConfig::Disabled => {
            let snapshot_manager = SnapshotDisabled {};
            Runner::start(server_manager, broker, snapshot_manager)
                .await
                .context("error in the advance runner main loop")
        }
    }
}
