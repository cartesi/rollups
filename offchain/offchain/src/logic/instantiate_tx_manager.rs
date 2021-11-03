use super::instantiate_block_subscriber::DescartesBlockSubscriber;
use crate::error::*;

use block_subscriber::BlockSubscriber;
use middleware_factory::{HttpProviderFactory, WsProviderFactory};
use tx_manager::config::TMConfig;
use tx_manager::{provider::Factory, TransactionManager};

use snafu::ResultExt;
use std::sync::Arc;

pub type DescartesTxManager = Arc<
    TransactionManager<
        Factory<HttpProviderFactory>,
        BlockSubscriber<WsProviderFactory>,
        String,
    >,
>;

pub async fn instantiate_tx_manager(
    http_endpoint: String,
    block_subscriber: DescartesBlockSubscriber,
    config: &TMConfig,
) -> Result<DescartesTxManager> {
    let middleware_factory = create_http_factory(http_endpoint)?;
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

    Ok(transaction_manager)
}

fn create_http_factory(
    http_endpoint: String,
) -> Result<Arc<HttpProviderFactory>> {
    HttpProviderFactory::new(http_endpoint.clone())
        .context(MiddlewareFactoryError {})
}
