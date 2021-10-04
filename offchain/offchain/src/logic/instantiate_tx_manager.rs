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

pub type DescartesTxManager = Arc<
    TransactionManager<
        Factory<HttpProviderFactory>,
        BlockSubscriber<WsProviderFactory>,
        String,
    >,
>;

pub type DescartesBlockSubscriber = Arc<BlockSubscriber<WsProviderFactory>>;

pub async fn instantiate_tx_manager(
    config: &crate::config::ApplicationConfig,
) -> Result<(
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
    DescartesBlockSubscriber,
    DescartesTxManager,
    DescartesV2Impl<<HttpProviderFactory as MiddlewareFactory>::Middleware>,
)> {
    let middleware_factory = create_http_factory(config)?;
    let descartesv2_contract = DescartesV2Impl::new(
        config
            .basic_config
            .contracts
            .get("DescartesV2Impl")
            .unwrap()
            .clone(),
        middleware_factory.current().await,
    );
    let factory = Factory::new(
        Arc::clone(&middleware_factory),
        config.tm_config.transaction_timeout,
    );
    let (block_subscriber, handle) = create_block_subscriber(config).await?;
    let transaction_manager = Arc::new(TransactionManager::new(
        factory,
        Arc::clone(&block_subscriber),
        config.tm_config.max_retries,
        config.tm_config.max_delay,
    ));
    Ok((
        handle,
        block_subscriber,
        transaction_manager,
        descartesv2_contract,
    ))
}

async fn create_ws_factory(
    config: &crate::config::ApplicationConfig,
) -> Result<Arc<WsProviderFactory>> {
    WsProviderFactory::new(
        config.basic_config.ws_url.clone().unwrap().clone(),
        config.tm_config.max_retries,
        config.tm_config.max_delay,
    )
    .await
    .context(MiddlewareFactoryError {})
}

fn create_http_factory(
    config: &crate::config::ApplicationConfig,
) -> Result<Arc<HttpProviderFactory>> {
    HttpProviderFactory::new(config.basic_config.url.clone())
        .context(MiddlewareFactoryError {})
}

async fn create_block_subscriber(
    config: &crate::config::ApplicationConfig,
) -> Result<(
    Arc<BlockSubscriber<WsProviderFactory>>,
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
)> {
    let factory = create_ws_factory(config).await?;
    Ok(BlockSubscriber::create_and_start(
        factory,
        config.bs_config.subscriber_timeout,
        config.bs_config.max_retries,
        config.bs_config.max_delay,
    ))
}
