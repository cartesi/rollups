use super::error::*;

use dispatcher::block_subscriber::{BlockSubscriber, BlockSubscriberHandle};
use dispatcher::middleware_factory::{
    HttpProviderFactory, MiddlewareFactory, WsProviderFactory,
};
use transaction_manager::{provider::Factory, TransactionManager};

use snafu::ResultExt;
use std::sync::Arc;

pub struct Config {
    http_endpoint: String,
    ws_endpoint: String,
    max_retries: usize,
    max_delay: std::time::Duration,

    call_timeout: std::time::Duration,
    subscriber_timeout: std::time::Duration,
}

pub type DescartesTxManager = Arc<
    TransactionManager<
        Factory<HttpProviderFactory>,
        BlockSubscriber<WsProviderFactory>,
        (),
    >,
>;

pub type DescartesBlockSubscriber = Arc<BlockSubscriber<WsProviderFactory>>;

pub async fn instantiate_tx_manager(
    config: &Config,
) -> Result<(
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
    DescartesBlockSubscriber,
    DescartesTxManager,
)> {
    let middleware_factory = create_http_factory(config)?;
    let factory = Factory::new(middleware_factory, config.call_timeout);
    let (block_subscriber, handle) = create_block_subscriber(config).await?;
    let transaction_manager = Arc::new(TransactionManager::new(
        factory,
        Arc::clone(&block_subscriber),
        config.max_retries,
        config.max_delay,
    ));
    Ok((handle, block_subscriber, transaction_manager))
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
