use offchain_core::ethers;

use crate::config::ApplicationConfig;
use crate::error::*;
use crate::fold::*;

use state_fold::Access;

use ethers::core::types::{Address, U64};
use ethers::providers::{Http, Provider};

use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

pub type DescartesAccess = Access<Provider<Http>>;

pub fn instantiate_state_fold(
    config: &ApplicationConfig,
) -> Result<DescartesStateFold<DescartesAccess>> {
    let access = create_access(config)?;
    let state_fold = create_descartes_state_fold(access, &config.sf_config);
    Ok(state_fold)
}

fn create_provider(url: String) -> Result<Arc<Provider<Http>>> {
    Ok(Arc::new(
        Provider::<Http>::try_from(url.clone()).context(UrlParseError {})?,
    ))
}

pub fn create_access(
    config: &ApplicationConfig,
) -> Result<Arc<DescartesAccess>> {
    let provider = create_provider(config.basic_config.url.clone())?;

    Ok(Arc::new(Access::new(
        provider,
        config.sf_config.genesis_block,
        config.sf_config.query_limit_error_codes.clone(),
        config.sf_config.concurrent_events_fetch,
    )))
}
