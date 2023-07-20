// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use rollups_events::DAppMetadata;
use std::error::Error;
use tracing::trace;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use authority_claimer::{
    claimer::{AuthorityClaimer, DefaultAuthorityClaimer},
    config::Config,
    listener::DefaultBrokerListener,
    metrics::AuthorityClaimerMetrics,
    sender::TxManagerClaimSender,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Getting the configuration.
    let config = Config::new().map_err(Box::new)?;

    tracing::info!(?config, "starting authority-claimer");

    // Settin up the logging environment.
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Creating the metrics and health server.
    let metrics = AuthorityClaimerMetrics::new();
    let http_server_handle =
        http_server::start(config.http_server_config, metrics.clone().into());

    let dapp_address = config.authority_claimer_config.dapp_address;
    let dapp_metadata = DAppMetadata {
        chain_id: config.authority_claimer_config.txm_config.chain_id,
        dapp_address: rollups_events::Address::new(dapp_address.into()),
    };

    // Creating the default broker listener.
    trace!("Creating the broker listener");
    let default_broker_listener = DefaultBrokerListener::new(
        config.authority_claimer_config.broker_config.clone(),
        dapp_metadata.clone(),
        metrics.clone(),
    )
    .map_err(Box::new)?;

    // Creating the transaction manager claim sender.
    trace!("Creating the claim sender");
    let tx_manager_claim_sender =
        TxManagerClaimSender::new(dapp_metadata, metrics).map_err(Box::new)?;

    // Creating the claimer loop.
    let authority_claimer = DefaultAuthorityClaimer::new();
    let claimer_handle = authority_claimer
        .start(default_broker_listener, tx_manager_claim_sender);

    // Starting the HTTP server and the claimer loop.
    tokio::select! {
        ret = http_server_handle => { ret.map_err(Box::new)? }
        ret = claimer_handle     => { ret.map_err(Box::new)? }
    };

    unreachable!()
}
