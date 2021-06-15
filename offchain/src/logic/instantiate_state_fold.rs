use super::error::*;
use super::fold::*;
use dispatcher::state_fold::Access;

use ethers::core::types::{Address, U64};
use ethers::providers::{Http, Provider};

use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

pub struct Config {
    safety_margin: usize,
    input_contract_address: Address, // TODO: read from contract.
    descartes_contract_address: Address,

    provider_http_url: String,
    genesis_block: U64,
    query_limit_error_codes: Vec<i32>,
    concurrent_events_fetch: usize,
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

fn create_access(config: &Config) -> Result<Arc<DescartesAccess>> {
    let provider = create_provider(config.provider_http_url)?;

    Ok(Arc::new(Access::new(
        provider,
        config.genesis_block,
        config.query_limit_error_codes,
        config.concurrent_events_fetch,
    )))
}

impl From<&Config> for SetupConfig {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        SetupConfig {
            safety_margin: config.safety_margin,
            input_contract_address: config.input_contract_address,
            descartes_contract_address: config.descartes_contract_address,
        }
    }
}
