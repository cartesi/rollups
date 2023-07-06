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

mod config;
mod controller;
mod conversions;
mod driver;
mod grpc;
mod hash;
mod http;
mod merkle_tree;
mod model;
mod proofs;

use clap::Parser;
use futures_util::FutureExt;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use std::time::Duration;
use tokio::sync::oneshot;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use config::Config;
use controller::Controller;

fn log_result<T, E: std::error::Error>(name: &str, result: Result<T, E>) {
    let prefix = format!("http {} terminated ", name);
    match result {
        Ok(_) => tracing::info!("{} successfully", prefix),
        Err(e) => tracing::warn!("{} with error: {}", prefix, e),
    };
}

#[actix_web::main]
async fn main() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = Config::parse();
    tracing::info!("{:#?}", config);

    let controller =
        Controller::new(Duration::from_millis(config.finish_timeout));
    let http_service_running = Arc::new(AtomicBool::new(true));
    let (grpc_shutdown_tx, grpc_shutdown_rx) = oneshot::channel::<()>();
    let grpc_service = {
        let controller = controller.clone();
        let config = config.clone();
        let shutdown = grpc_shutdown_rx.map(|_| ());
        let http_service_running = http_service_running.clone();
        tokio::spawn(async move {
            log_result(
                "gRPC service",
                grpc::start_service(&config, controller.clone(), shutdown)
                    .await,
            );
            if http_service_running.load(Ordering::Relaxed) {
                panic!("gRPC service terminated before shutdown signal");
            }
        })
    };

    // We run the actix-web in the main thread because it handles the SIGINT
    let host_runner_handle = http::start_services(&config, controller.clone());
    if config.health_check_config.enabled {
        let health_handle =
            http_health_check::start(config.health_check_config.port);

        tokio::select! {
            result = health_handle => {
                log_result("http health check", result);
            }
            result = host_runner_handle => {
                log_result("http service", result);
            }
        }
    } else {
        log_result("http service", host_runner_handle.await);
    }
    http_service_running.store(false, Ordering::Relaxed);

    // Shutdown the other services
    if let Err(e) = controller.shutdown().await.await {
        tracing::error!("failed to shutdown controller ({})", e);
    }
    if let Err(_) = grpc_shutdown_tx.send(()) {
        tracing::error!("failed to send the shutdown signal to grpc");
    }
    if let Err(e) = grpc_service.await {
        tracing::error!("failed to shutdown the grpc service ({})", e);
    }
}
