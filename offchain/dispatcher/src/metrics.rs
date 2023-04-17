use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{routing::get, Router};
use prometheus_client::{
    encoding::text::encode, metrics::counter::Counter, registry::Registry,
};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

/// This trait must be implemented by the struct that aggregates
/// the programs's metrics.
#[async_trait]
pub trait Metrics {
    /// Returns a registry initialized with the program's metrics.
    fn registry(&self) -> Registry;

    /// Starts the "/metrics" endpoint for Prometheus to poll from.
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

// ----------------------------------------------------------------------------
//
// ----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct DispatcherMetrics {
    pub claims_sent_total: Counter,
    pub advance_inputs_sent_total: Counter,
    pub finish_epochs_sent_total: Counter,
}

impl Metrics for DispatcherMetrics {
    fn registry(&self) -> Registry {
        let mut registry = Registry::default();
        registry.register(
            "claims_sent",
            "Counts the number of claims sent",
            self.claims_sent_total.clone(),
        );
        registry.register(
            "advance_inputs_sent",
            "Counts the number of advance inputs sent",
            self.advance_inputs_sent_total.clone(),
        );
        registry.register(
            "finish_epochs_sent",
            "Counts the number of finish epochs sent",
            self.finish_epochs_sent_total.clone(),
        );
        registry
    }
}

// ----------------------------------------------------------------------------
//
// ----------------------------------------------------------------------------

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct MetricsCLIConfig {
    #[arg(long, env, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, env, default_value_t = 9091)]
    pub port: u16,
}

// Keeping standards.
pub type MetricsConfig = MetricsCLIConfig;

impl MetricsConfig {
    pub fn initialize(cli: MetricsCLIConfig) -> MetricsConfig {
        cli
    }
}
