// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use clap::Parser;

use inspect_server::config::CLIConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .target(env_logger::fmt::Target::Stdout)
    .init();
    let config = CLIConfig::parse().into();

    inspect_server::run(config).await.map_err(|e| e.into())
}
