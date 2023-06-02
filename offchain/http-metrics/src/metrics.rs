use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{routing::get, Router};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

/// This trait must be implemented by the struct that aggregates
/// the programs's metrics.
#[async_trait]
pub trait Metrics {
    /// Must return a registry initialized with the program's metrics.
    fn registry(&self) -> Registry;

    /// Starts the "/metrics" endpoint using axum for Prometheus to poll from.
    async fn run(&self, host: String, port: u16) -> JoinHandle<Result<()>> {
        let registry = self.registry();
        tokio::spawn(async move { run(host, port, registry).await })
    }
}

async fn run(host: String, port: u16, registry: Registry) -> Result<()> {
    tracing::info!(
        "Starting metrics endpoint at http://{}:{}/metrics",
        host,
        port
    );

    let host = host.parse().context("could not parse host address")?;
    let addr = SocketAddr::new(host, port);
    let registry = Arc::new(Mutex::new(registry));
    let router = Router::new().route(
        "/metrics",
        get(|| async move {
            let mut buffer = String::new();
            let registry = registry.lock().unwrap();
            encode(&mut buffer, &registry).unwrap();
            buffer
        }),
    );

    Ok(axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?)
}
