// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

mod server_manager;

use futures_util::FutureExt;
use std::future::Future;
use tonic::transport::Server;
use tonic_health::pb::health_server::{Health, HealthServer};

use grpc_interfaces::cartesi_server_manager::server_manager_server::ServerManagerServer;
use server_manager::ServerManagerService;

use crate::config::Config;
use crate::controller::Controller;

/// Create the grpc healthcheck for the host-runner
///
/// Since the host-runner doesn't rely on any other service to function, it is always
/// healthy.
async fn create_health_service() -> HealthServer<impl Health> {
    let (mut health_reporter, health_service) =
        tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ServerManagerServer<ServerManagerService>>()
        .await;
    health_service
}

pub async fn start_service<F: Future<Output = ()>>(
    config: &Config,
    controller: Controller,
    signal: F,
) -> Result<(), tonic::transport::Error> {
    let addr = format!(
        "{}:{}",
        config.grpc_server_manager_address, config.grpc_server_manager_port
    )
    .parse()
    .expect("invalid config");
    let service = ServerManagerService::new(controller);
    Server::builder()
        .add_service(create_health_service().await)
        .add_service(ServerManagerServer::new(service))
        .serve_with_shutdown(addr, signal.map(|_| ()))
        .await
}
