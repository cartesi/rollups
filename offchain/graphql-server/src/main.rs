// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use graphql_server::CLIConfig;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracing_format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(true)
        .with_target(true)
        .with_ansi(false)
        .compact();
    if std::env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_max_level(LevelFilter::INFO)
            .init();
    }

    let config = CLIConfig::parse().into();

    graphql_server::run(config).await.map_err(|e| e.into())
}
