use offchain_core::ethers;

use crate::contracts::descartesv2_contract::DescartesV2Impl;
use crate::error::*;

use block_subscriber::{BlockSubscriber, BlockSubscriberHandle};
use middleware_factory::{
    HttpProviderFactory, MiddlewareFactory, WsProviderFactory,
};
use tx_manager::{provider::Factory, TransactionManager};

use ethers::core::types::Address;
use snafu::ResultExt;
use std::sync::Arc;

pub struct Config {
    pub http_endpoint: String,
    pub ws_endpoint: String,
    pub max_retries: usize,
    pub max_delay: std::time::Duration,

    pub call_timeout: std::time::Duration,
    pub subscriber_timeout: std::time::Duration,

    pub descartes_contract_address: Address,
}

pub type DescartesTxManager = Arc<
    TransactionManager<
        Factory<HttpProviderFactory>,
        BlockSubscriber<WsProviderFactory>,
        String,
    >,
>;

pub type DescartesBlockSubscriber = Arc<BlockSubscriber<WsProviderFactory>>;

pub async fn instantiate_tx_manager(
    config: &Config,
) -> Result<(
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
    DescartesBlockSubscriber,
    DescartesTxManager,
    DescartesV2Impl<<HttpProviderFactory as MiddlewareFactory>::Middleware>,
)> {
    let middleware_factory = create_http_factory(config)?;
    let descartesv2_contract = DescartesV2Impl::new(
        config.descartes_contract_address,
        middleware_factory.current().await,
    );
    let factory =
        Factory::new(Arc::clone(&middleware_factory), config.call_timeout);
    let (block_subscriber, handle) = create_block_subscriber(config).await?;
    let transaction_manager = Arc::new(TransactionManager::new(
        factory,
        Arc::clone(&block_subscriber),
        config.max_retries,
        config.max_delay,
    ));
    Ok((
        handle,
        block_subscriber,
        transaction_manager,
        descartesv2_contract,
    ))
}

async fn create_ws_factory(config: &Config) -> Result<Arc<WsProviderFactory>> {
    WsProviderFactory::new(
        config.ws_endpoint.clone(),
        config.max_retries,
        config.max_delay,
    )
    .await
    .context(MiddlewareFactoryError {})
}

fn create_http_factory(config: &Config) -> Result<Arc<HttpProviderFactory>> {
    HttpProviderFactory::new(config.http_endpoint.clone())
        .context(MiddlewareFactoryError {})
}

async fn create_block_subscriber(
    config: &Config,
) -> Result<(
    Arc<BlockSubscriber<WsProviderFactory>>,
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
)> {
    let factory = create_ws_factory(config).await?;
    Ok(BlockSubscriber::create_and_start(
        factory,
        config.subscriber_timeout,
        config.max_retries,
        config.max_delay,
    ))
}
