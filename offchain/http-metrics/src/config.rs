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
