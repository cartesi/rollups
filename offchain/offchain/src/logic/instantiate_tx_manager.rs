use super::instantiate_block_subscriber::RollupsBlockSubscriber;
use crate::error::*;

use block_subscriber::BlockSubscriber;
use middleware_factory::{
    HttpProviderFactory, LocalSignerFactory, WsProviderFactory,
};
use offchain_core::ethers::{
    signers::{coins_bip39::English, MnemonicBuilder, Signer},
    types::Address,
};
use tx_manager::config::TMConfig;
use tx_manager::{provider::Factory, TransactionManager};

use snafu::ResultExt;
use std::sync::Arc;

pub type RollupsSignerFactory = LocalSignerFactory<HttpProviderFactory>;

pub type RollupsTxManager = Arc<
    TransactionManager<
        Factory<RollupsSignerFactory>,
        BlockSubscriber<WsProviderFactory>,
        String,
    >,
>;

pub async fn instantiate_tx_manager(
    http_endpoint: String,
    mnemonic: String,
    block_subscriber: RollupsBlockSubscriber,
    config: &TMConfig,
) -> Result<(RollupsTxManager, Address)> {
    let (middleware_factory, sender) =
        create_signer_factory(http_endpoint, mnemonic).await?;
    let factory = Factory::new(
        Arc::clone(&middleware_factory),
        config.transaction_timeout,
    );

    let transaction_manager = Arc::new(TransactionManager::new(
        factory,
        block_subscriber,
        config.max_retries,
        config.max_delay,
    ));

    Ok((transaction_manager, sender))
}

async fn create_signer_factory(
    http_endpoint: String,
    mnemonic: String,
) -> Result<(Arc<RollupsSignerFactory>, Address)> {
    let http_factory = create_http_factory(http_endpoint)?;

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic.as_str())
        .build()
        .context(MnemonicError)?;

    let address = wallet.address();

    let signer_factory = LocalSignerFactory::new(http_factory, wallet)
        .await
        .context(MiddlewareFactoryError)?;

    Ok((signer_factory, address))
}

fn create_http_factory(
    http_endpoint: String,
) -> Result<Arc<HttpProviderFactory>> {
    HttpProviderFactory::new(http_endpoint.clone())
        .context(MiddlewareFactoryError {})
}
