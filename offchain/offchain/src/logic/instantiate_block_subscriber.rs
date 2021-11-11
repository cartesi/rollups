use crate::error::*;

use block_subscriber::{
    config::BSConfig, BlockSubscriber, BlockSubscriberHandle,
};
use middleware_factory::{MiddlewareFactory, WsProviderFactory};

use snafu::ResultExt;
use std::sync::Arc;

pub type RollupsBlockSubscriber = Arc<BlockSubscriber<WsProviderFactory>>;

pub async fn instantiate_block_subscriber(
    ws_endpoint: String,
    config: &BSConfig,
) -> Result<(
    RollupsBlockSubscriber,
    BlockSubscriberHandle<<WsProviderFactory as MiddlewareFactory>::Middleware>,
)> {
    let factory = create_ws_factory(ws_endpoint, config).await?;
    Ok(BlockSubscriber::create_and_start(
        factory,
        config.subscriber_timeout,
        config.max_retries,
        config.max_delay,
    ))
}

async fn create_ws_factory(
    ws_endpoint: String,
    config: &BSConfig,
) -> Result<Arc<WsProviderFactory>> {
    WsProviderFactory::new(ws_endpoint, config.max_retries, config.max_delay)
        .await
        .context(MiddlewareFactoryError)
}
