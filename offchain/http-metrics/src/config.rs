use clap::Parser;
use std::net::AddrParseError;

use crate::MetricsServer;

#[derive(Clone, Debug, Parser)]
pub struct MetricsCLIConfig {
    #[arg(long, env, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, env, default_value_t = 9091)]
    pub port: u16,
}

impl TryFrom<MetricsCLIConfig> for MetricsServer {
    type Error = AddrParseError;

    fn try_from(config: MetricsCLIConfig) -> Result<Self, Self::Error> {
        Ok(MetricsServer {
            host: config.host.parse()?,
            port: config.port,
        })
    }
}
