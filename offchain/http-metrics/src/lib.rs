pub mod config;
mod metrics;

pub use metrics::Metrics;

pub use prometheus_client::registry::Registry;

// Re-exporting prometheus metrics.
// Add any other metrics to re-export here.
pub use prometheus_client::metrics::counter::Counter;
