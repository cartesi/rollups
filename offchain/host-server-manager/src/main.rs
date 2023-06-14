// Copyright 2021 Cartesi Pte. Ltd.
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

use futures_util::FutureExt;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::oneshot;

use config::Config;
use controller::Controller;

#[actix_web::main]
async fn main() {
    let config = Config::from_args();

    // Set the default log level to info
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&config.log_level))
        .init();
    log::info!("{:#?}", config);

    let controller = Controller::new(Duration::from_millis(config.finish_timeout));
    let http_service_running = Arc::new(AtomicBool::new(true));
    let (grpc_shutdown_tx, grpc_shutdown_rx) = oneshot::channel::<()>();
    let grpc_service = {
        let controller = controller.clone();
        let config = config.clone();
        let shutdown = grpc_shutdown_rx.map(|_| ());
        let http_service_running = http_service_running.clone();
        tokio::spawn(async move {
            match grpc::start_service(&config, controller.clone(), shutdown).await {
                Ok(_) => log::info!("grpc service terminated successfully"),
                Err(e) => log::warn!("grpc service terminated with error: {}", e),
            }
            if http_service_running.load(Ordering::Relaxed) {
                panic!("gRPC service terminated before shutdown signal");
            }
        })
    };

    // We run the actix-web in the main thread because it handles the SIGINT
    match http::start_services(&config, controller.clone()).await {
        Ok(_) => log::info!("http service terminated successfully"),
        Err(e) => log::warn!("http service terminated with error: {}", e),
    }
    http_service_running.store(false, Ordering::Relaxed);

    // Shutdown the other services
    if let Err(e) = controller.shutdown().await.await {
        log::error!("failed to shutdown controller ({})", e);
    }
    if let Err(_) = grpc_shutdown_tx.send(()) {
        log::error!("failed to send the shutdown signal to grpc");
    }
    if let Err(e) = grpc_service.await {
        log::error!("failed to shutdown the grpc service ({})", e);
    }
}
