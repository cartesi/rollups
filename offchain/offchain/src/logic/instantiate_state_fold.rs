use offchain_core::ethers;

use crate::error::*;
use crate::fold::*;

use state_fold::config::SFConfig;
use state_fold::Access;

use ethers::providers::{Http, Provider};

use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

pub type RollupsAccess = Access<Provider<Http>>;

pub fn instantiate_state_fold(
    config: &SFConfig,
    url: String,
) -> Result<RollupsStateFold<RollupsAccess>> {
    let access = create_access(config, url)?;
    let state_fold = create_rollups_state_fold(access, &config);
    Ok(state_fold)
}

fn create_provider(url: String) -> Result<Arc<Provider<Http>>> {
    Ok(Arc::new(
        Provider::<Http>::try_from(url.clone()).context(UrlParseError {})?,
    ))
}

pub fn create_access(
    config: &SFConfig,
    url: String,
) -> Result<Arc<RollupsAccess>> {
    let provider = create_provider(url)?;

    Ok(Arc::new(Access::new(
        provider,
        config.genesis_block,
        config.query_limit_error_codes.clone(),
        config.concurrent_events_fetch,
    )))
}
