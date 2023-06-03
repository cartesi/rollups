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

use anyhow::Result;
use dispatcher::metrics::DispatcherMetrics;
use http_metrics::MetricsServer;

// NOTE: doesn't support History upgradability.
// NOTE: doesn't support changing epoch_duration in the middle of things.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = dispatcher::config::DispatcherConfig::initialize_from_args()?;
    let hc_config = config.hc_config.clone();

    let health_handle = tokio::spawn(async move {
        dispatcher::http_health::start_health_check(
            &hc_config.host_address,
            hc_config.port,
        )
        .await
    });

    let (metrics, metrics_handle) = {
        let metrics = DispatcherMetrics::default();
        let metrics_host = config.metrics_config.host.clone();
        let metrics_port = config.metrics_config.port;
        let metrics_handle = metrics.run(metrics_host, metrics_port).await;
        (metrics, metrics_handle)
    };

    let dispatcher_handle = tokio::spawn(async move {
        dispatcher::main_loop::run(config, metrics).await
    });

    tokio::select! {
        ret = health_handle => {
            tracing::error!("HTTP health-check stopped: {:?}", ret);
            ret??;
        }

        ret = metrics_handle => {
            tracing::error!("HTTP metrics stopped: {:?}", ret);
            ret??;
        }

        ret = dispatcher_handle => {
            tracing::error!("Dispatcher stopped: {:?}", ret);
            ret??;
        }
    }

    Ok(())
}
