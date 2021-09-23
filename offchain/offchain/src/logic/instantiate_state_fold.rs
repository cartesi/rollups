use offchain_core::ethers;

use crate::error::*;
use crate::fold::*;

use state_fold::Access;

use ethers::core::types::{Address, U64};
use ethers::providers::{Http, Provider};

use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

pub struct Config {
    pub safety_margin: usize,
    pub input_contract_address: Address, // TODO: read from contract.
    pub output_contract_address: Address,
    pub descartes_contract_address: Address,

    pub provider_http_url: String,
    pub genesis_block: U64,
    pub query_limit_error_codes: Vec<i32>,
    pub concurrent_events_fetch: usize,
}

pub type DescartesAccess = Access<Provider<Http>>;

pub fn instantiate_state_fold(
    config: &Config,
) -> Result<DescartesStateFold<DescartesAccess>> {
    let access = create_access(config)?;
    let setup_config = SetupConfig::from(config);
    let state_fold = create_descartes_state_fold(access, &setup_config);
    Ok(state_fold)
}

fn create_provider(url: String) -> Result<Arc<Provider<Http>>> {
    Ok(Arc::new(
        Provider::<Http>::try_from(url.clone()).context(UrlParseError {})?,
    ))
}

pub fn create_access(config: &Config) -> Result<Arc<DescartesAccess>> {
    let provider = create_provider(config.provider_http_url.clone())?;

    Ok(Arc::new(Access::new(
        provider,
        config.genesis_block,
        config.query_limit_error_codes.clone(),
        config.concurrent_events_fetch,
    )))
}

impl From<&Config> for SetupConfig {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        SetupConfig {
            safety_margin: config.safety_margin,
        }
    }
}
