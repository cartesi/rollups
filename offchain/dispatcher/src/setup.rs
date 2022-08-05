use crate::{
    config::DispatcherConfig,
    machine::rollup_server::{Config as MMConfig, MachineManager},
    rollups_dispatcher::RollupsDispatcher,
    tx_sender::{BulletproofTxSender, TxSender},
};

use state_client_lib::{
    config::SCConfig, error::StateServerError, BlockServer,
    GrpcStateFoldClient, StateServer,
};
use state_fold_types::{
    ethers::{
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{coins_bip39::English, MnemonicBuilder, Signer},
        types::Address,
    },
    BlockStreamItem,
};
use tx_manager::{
    config::TxManagerConfig, database::FileSystemDatabase,
    gas_oracle::ETHGasStationOracle, Priority, TimeConfiguration,
    TransactionManager,
};

use types::{
    fee_manager::FeeIncentiveStrategy, rollups::RollupsState,
    rollups_initial_state::RollupsInitialState,
};

use anyhow::Result;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

const BUFFER_LEN: usize = 256;

pub async fn create_state_server(
    config: &SCConfig,
) -> Result<
    impl StateServer<InitialState = RollupsInitialState, State = RollupsState>
        + BlockServer,
> {
    let channel = Channel::from_shared(config.grpc_endpoint.to_owned())?
        .connect()
        .await?;

    Ok(GrpcStateFoldClient::new_from_channel(channel))
}

pub async fn create_block_subscription(
    client: &impl BlockServer,
    confirmations: usize,
) -> Result<
    impl Stream<Item = Result<BlockStreamItem, StateServerError>>
        + std::marker::Unpin,
> {
    let s = client.subscribe_blocks(confirmations).await?;

    let s = {
        use futures::StreamExt;
        s.ready_chunks(BUFFER_LEN)
    };

    let s = s.filter_map(|mut x| {
        if x.len() == BUFFER_LEN {
            None
        } else {
            let a = x.pop();
            a
        }
    });

    Ok(s)
}

pub async fn create_tx_sender(
    config: &TxManagerConfig,
    dapp_contract_address: Address,
    priority: Priority,
) -> Result<impl TxSender> {
    let tx_manager = {
        let provider = {
            let provider = Provider::<Http>::try_from(
                config.provider_http_endpoint.to_owned(),
            )?;

            let wallet = MnemonicBuilder::<English>::default()
                .phrase(config.mnemonic.as_str())
                .build()?
                .with_chain_id(config.chain_id);

            assert_eq!(
                wallet.address(),
                config.sender,
                "mnemonic public key does not match sender"
            );

            SignerMiddleware::new(provider, wallet)
        };

        let database = FileSystemDatabase::new(config.database_path.to_owned());

        let (tx_manager, _) = TransactionManager::new(
            provider,
            None::<ETHGasStationOracle>,
            database,
            config.chain_id.into(),
            TimeConfiguration::default(),
        )
        .await?;

        tx_manager
    };

    Ok(BulletproofTxSender::new(
        tx_manager,
        config.default_confirmations,
        priority,
        config.sender,
        dapp_contract_address,
    ))
}

pub async fn create_dispatcher(
    config: &DispatcherConfig,
    sender: Address,
) -> Result<RollupsDispatcher<MachineManager>> {
    let machine_manager = {
        let mm_config = MMConfig::new_with_default(
            config.mm_config.endpoint.to_owned(),
            config.mm_config.session_id.to_owned(),
        );

        MachineManager::new(mm_config).await?
    };

    let fee_incentive_strategy = FeeIncentiveStrategy {
        minimum_required_fee: config.minimum_required_fee,
        num_buffer_epochs: config.num_buffer_epochs,
        num_claims_trigger_redeem: config.num_claims_trigger_redeem,
    };

    Ok(RollupsDispatcher::new(
        machine_manager,
        fee_incentive_strategy,
        sender,
    ))
}
