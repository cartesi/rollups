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

use grpc_interfaces::cartesi_machine::{
    ConcurrencyConfig, MachineRuntimeConfig,
};
use grpc_interfaces::cartesi_server_manager::{CyclesConfig, DeadlineConfig};
use server_manager_broker_proxy::config::{
    BrokerConfig, Config, FSManagerConfig, HealthCheckConfig, ProxyConfig,
    ServerManagerConfig, SnapshotConfig,
};
use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct ProxyFixture {
    config: Config,
    handler: RefCell<Option<JoinHandle<anyhow::Result<()>>>>,
}

impl ProxyFixture {
    pub async fn setup(
        server_manager_endpoint: String,
        session_id: String,
        redis_endpoint: String,
        chain_id: u64,
        dapp_contract_address: [u8; 20],
        snapshot_dir: &Path,
    ) -> Self {
        let runtime_config = MachineRuntimeConfig {
            concurrency: Some(ConcurrencyConfig {
                update_merkle_tree: 0,
            }),
        };

        let deadline_config = DeadlineConfig {
            checkin: 1000 * 5,
            advance_state: 1000 * 60 * 3,
            advance_state_increment: 1000 * 10,
            inspect_state: 1000 * 60 * 3,
            inspect_state_increment: 1000 * 10,
            machine: 1000 * 60 * 5,
            store: 1000 * 60 * 3,
            fast: 1000 * 5,
        };

        let cycles_config = CyclesConfig {
            max_advance_state: u64::MAX >> 2,
            advance_state_increment: 1 << 22,
            max_inspect_state: u64::MAX >> 2,
            inspect_state_increment: 1 << 22,
        };

        let server_manager_config = ServerManagerConfig {
            server_manager_endpoint,
            session_id,
            pending_inputs_sleep_duration: 1000,
            pending_inputs_max_retries: 10,
            runtime_config,
            deadline_config,
            cycles_config,
        };

        let broker_config = BrokerConfig {
            redis_endpoint,
            chain_id,
            dapp_contract_address,
            consume_timeout: 100,
        };

        let snapshot_config = SnapshotConfig::FileSystem(FSManagerConfig {
            snapshot_dir: snapshot_dir.to_owned(),
            snapshot_latest: snapshot_dir.join("latest"),
        });

        let backoff_max_elapsed_duration = Duration::from_millis(1);

        let proxy_config = ProxyConfig {
            server_manager_config,
            broker_config,
            snapshot_config,
            backoff_max_elapsed_duration,
        };

        let health_check_config = HealthCheckConfig {
            health_check_address: "0.0.0.0".to_string(),
            health_check_port: 0,
        };

        let config = Config {
            proxy_config,
            health_check_config,
        };

        let handler = RefCell::new(Some(start_proxy(config.clone())));
        Self { config, handler }
    }

    /// Wait until the proxy exists with an error
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn wait_err(&self) -> anyhow::Error {
        tracing::trace!("waiting for proxy error");
        let handler = self.handler.replace(None);
        handler
            .expect("handler not found")
            .await
            .expect("failed to wait for handler")
            .expect_err("proxy should exit with an error")
    }

    /// Abort the current proxy proxy, wait it to finish and start another one
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn restart(&self) {
        tracing::trace!("restartin proxy");
        let handler = self.handler.replace(None).expect("handler not found");
        handler.abort();
        handler.await.expect_err("proxy finished before abort");
        let new_handler = start_proxy(self.config.clone());
        self.handler.replace(Some(new_handler));
    }
}

fn start_proxy(config: Config) -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(async move {
        let output = server_manager_broker_proxy::run(config).await;
        tracing::error!(?output, "proxy exited");
        output
    })
}

impl Drop for ProxyFixture {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.borrow().as_ref() {
            handler.abort();
        }
    }
}
