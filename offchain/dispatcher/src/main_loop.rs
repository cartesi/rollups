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
use tracing::{error, instrument, warn};

#[instrument(level = "trace")]
pub async fn run(config: DispatcherConfig) -> Result<()> {
    let state_server = create_state_server(&config.sc_config).await?;

    let block_subscription = create_block_subscription(
        &state_server,
        config.sc_config.default_confirmations,
    )
    .await?;

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

    let dispatcher =
        create_dispatcher(&config, config.tx_config.sender).await?;

    main_loop(
        block_subscription,
        &state_server,
        initial_state,
        dispatcher,
        tx_sender,
    )
    .await
}

#[instrument(level = "trace", skip(client, block_subscription))]
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
                tx_sender = process_block(
                    &b,
                    client,
                    &initial_state,
                    &dispatcher,
                    tx_sender,
                )
                .await?
            }

            Some(Ok(BlockStreamItem::Reorg(_))) => {
                error!("Deep blockchain reorg, bailing");
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

#[instrument(level = "trace", skip(client))]
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
    let state = client.query_state(initial_state, block.hash).await?;
    dispatcher.react(state, tx_sender).await
}
