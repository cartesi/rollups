use crate::{
    config::DispatcherConfig,
    drivers::{blockchain::BlockchainDriver, machine::MachineDriver, Context},
    machine::{rollups_broker::BrokerFacade, BrokerReceive, BrokerSend},
    setup::{
        create_block_subscription, create_context, create_state_server,
        create_tx_sender,
    },
    tx_sender::TxSender,
};

use eth_tx_manager::transaction::Priority;
use rollups_events::{Address, DAppMetadata};
use state_client_lib::{error::StateServerError, StateServer};
use state_fold_types::{Block, BlockStreamItem};
use types::foldables::authority::rollups::{RollupsInitialState, RollupsState};

use anyhow::{bail, Result};
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, error, instrument, trace, warn};

#[instrument(level = "trace", skip_all)]
pub async fn run(config: DispatcherConfig) -> Result<()> {
    debug!("Setting up dispatcher with config: {:?}", config);

    trace!("Creating transaction manager");
    let tx_sender = create_tx_sender(
        &config.tx_config,
        config.rollups_deployment.authority_address,
        Priority::Normal,
    )
    .await?;

    trace!("Creating state-server connection");
    let state_server = create_state_server(&config.sc_config).await?;

    trace!("Starting block subscription with confirmations");
    let block_subscription = create_block_subscription(
        &state_server,
        config.sc_config.default_confirmations,
    )
    .await?;

    trace!("Creating broker connection");
    let broker = BrokerFacade::new(
        config.broker_config.clone(),
        DAppMetadata {
            chain_id: config.tx_config.chain_id,
            dapp_address: Address::new(
                config.dapp_deployment.dapp_address.into(),
            ),
        },
    )
    .await?;

    trace!("Creating context");
    let context = create_context(&config, &state_server, &broker).await?;

    trace!("Creating machine driver and blockchain driver");
    let machine_driver =
        MachineDriver::new(config.dapp_deployment.dapp_address);
    let blockchain_driver =
        BlockchainDriver::new(config.dapp_deployment.dapp_address);

    let initial_state = RollupsInitialState {
        history_address: config.rollups_deployment.history_address,
        input_box_address: config.rollups_deployment.input_box_address,
    };

    trace!("Entering main loop...");
    main_loop(
        block_subscription,
        state_server,
        initial_state,
        context,
        machine_driver,
        blockchain_driver,
        broker,
        tx_sender,
    )
    .await
}

#[instrument(level = "trace", skip_all)]
async fn main_loop<TS: TxSender + Sync + Send>(
    mut block_subscription: impl Stream<Item = Result<BlockStreamItem, StateServerError>>
        + Send
        + std::marker::Unpin,

    client: impl StateServer<
        InitialState = RollupsInitialState,
        State = RollupsState,
    >,
    initial_state: RollupsInitialState,

    mut context: Context,
    mut machine_driver: MachineDriver,
    mut blockchain_driver: BlockchainDriver,

    broker: impl BrokerSend + BrokerReceive,

    mut tx_sender: TS,
) -> Result<()> {
    loop {
        match block_subscription.next().await {
            Some(Ok(BlockStreamItem::NewBlock(b))) => {
                // Normal operation, react on newest block.
                trace!(
                    "Received block number {} and hash {:?}, parent: {:?}",
                    b.number,
                    b.hash,
                    b.parent_hash
                );
                tx_sender = process_block(
                    &b,
                    &client,
                    &initial_state,
                    &mut context,
                    &mut machine_driver,
                    &mut blockchain_driver,
                    &broker,
                    tx_sender,
                )
                .await?;
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
                warn!(
                    "Subscription returned error `{}`; waiting for next block...",
                    e
                );
            }

            None => {
                bail!("Subscription closed");
            }
        }
    }
}

#[instrument(level = "trace", skip_all)]
async fn process_block<TS: TxSender + Sync + Send>(
    block: &Block,

    client: &impl StateServer<
        InitialState = RollupsInitialState,
        State = RollupsState,
    >,
    initial_state: &RollupsInitialState,

    context: &mut Context,
    machine_driver: &mut MachineDriver,
    blockchain_driver: &mut BlockchainDriver,

    broker: &(impl BrokerSend + BrokerReceive),

    tx_sender: TS,
) -> Result<TS> {
    trace!("Querying rollup state");
    let state = client.query_state(initial_state, block.hash).await?;

    // Drive machine
    trace!("Reacting to state with `machine_driver`");
    machine_driver
        .react(context, &state.block, &state.state.input_box, broker)
        .await?;

    // Drive blockchain
    trace!("Reacting to state with `blockchain_driver`");
    blockchain_driver
        .react(&state.state.history, broker, tx_sender)
        .await
}
