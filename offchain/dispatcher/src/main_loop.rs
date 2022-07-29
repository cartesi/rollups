use crate::{
    config::DispatcherConfig,
    machine::MachineInterface,
    rollups_dispatcher::RollupsDispatcher,
    setup::{
        create_block_subscription, create_dispatcher, create_state_server,
        create_tx_sender,
    },
    tx_sender::TxSender,
};

use state_client_lib::{error::StateServerError, StateServer};
use state_fold_types::{Block, BlockStreamItem};
use tx_manager::transaction::Priority;

use types::{
    rollups::RollupsState, rollups_initial_state::RollupsInitialState,
};

use anyhow::{bail, Result};
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, error, instrument, trace, warn};

#[instrument(level = "trace", skip_all)]
pub async fn run(config: DispatcherConfig) -> Result<()> {
    debug!("Setting up dispatcher with config: {:?}", config);

    trace!("Creating state-server connection");
    let state_server = create_state_server(&config.sc_config).await?;

    trace!("Starting block subscription with confirmations");
    let block_subscription = create_block_subscription(
        &state_server,
        config.sc_config.default_confirmations,
    )
    .await?;

    trace!("Creating transaction manager");
    let tx_sender = create_tx_sender(
        &config.tx_config,
        config.dapp_contract_address,
        Priority::Normal,
    )
    .await?;

    let initial_state = RollupsInitialState {
        dapp_contract_address: Arc::new(config.dapp_contract_address),
        initial_epoch: config.initial_epoch,
    };

    trace!("Creating dispatcher");
    let dispatcher =
        create_dispatcher(&config, config.tx_config.sender).await?;

    trace!("Entering main loop...");
    main_loop(
        block_subscription,
        &state_server,
        initial_state,
        dispatcher,
        tx_sender,
    )
    .await
}

#[instrument(level = "trace", skip_all)]
async fn main_loop<
    TS: TxSender + Sync + Send,
    MM: MachineInterface + Send + Sync,
>(
    mut block_subscription: impl Stream<Item = Result<BlockStreamItem, StateServerError>>
        + std::marker::Unpin,

    client: &impl StateServer<
        InitialState = RollupsInitialState,
        State = RollupsState,
    >,

    initial_state: RollupsInitialState,

    dispatcher: RollupsDispatcher<MM>,

    mut tx_sender: TS,
) -> Result<()> {
    loop {
        match block_subscription.next().await {
            Some(Ok(BlockStreamItem::NewBlock(b))) => {
                trace!(
                    "Received block number {} and hash {:?}, parent: {:?}",
                    b.number,
                    b.hash,
                    b.parent_hash
                );
                tx_sender = process_block(
                    &b,
                    client,
                    &initial_state,
                    &dispatcher,
                    tx_sender,
                )
                .await?
            }

            Some(Ok(BlockStreamItem::Reorg(bs))) => {
                error!(
                    "Deep blockchain reorg of {} blocks; new latest has number {:?}, hash {:?}, and parent {:?}",
                    bs.len(),
                    bs.last().map(|b| b.number),
                    bs.last().map(|b| b.hash),
                    bs.last().map(|b| b.parent_hash)
                );
                error!("Bailing...");
                bail!("Deep blockchain reorg");
            }

            Some(Err(e)) => {
                warn!("Subscription returned error `{}`; waiting for next block...", e);
            }

            None => {
                bail!("Subscription closed");
            }
        }
    }
}

#[instrument(level = "trace", skip_all)]
async fn process_block<
    TS: TxSender + Sync + Send,
    MM: MachineInterface + Send + Sync,
>(
    block: &Block,

    client: &impl StateServer<
        InitialState = RollupsInitialState,
        State = RollupsState,
    >,

    initial_state: &RollupsInitialState,

    dispatcher: &RollupsDispatcher<MM>,

    tx_sender: TS,
) -> Result<TS> {
    trace!("Querying state");
    let state = client.query_state(initial_state, block.hash).await?;

    trace!("Reacting to state");
    dispatcher.react(state, tx_sender).await
}
