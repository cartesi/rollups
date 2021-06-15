use super::error::*;
use super::fold::types::*;
use super::instantiate_state_fold::{self, instantiate_state_fold};
use super::instantiate_tx_manager::{
    self, instantiate_tx_manager, DescartesTxManager,
};

use dispatcher::block_subscriber::{
    BlockSubscriber, BlockSubscriberHandle, NewBlockSubscriber,
};
use dispatcher::state_fold::types::BlockState;

use ethers::core::types::{Address, U256, U64};
use snafu::ResultExt;

pub struct Config {
    safety_margin: usize,
    input_contract_address: Address, // TODO: read from contract.
    descartes_contract_address: Address,

    provider_http_url: String,
    genesis_block: U64,
    query_limit_error_codes: Vec<i32>,
    concurrent_events_fetch: usize,

    http_endpoint: String,
    ws_endpoint: String,
    max_retries: usize,
    max_delay: std::time::Duration,

    call_timeout: std::time::Duration,
    subscriber_timeout: std::time::Duration,

    initial_epoch: U256,
}

async fn main_loop(config: &Config) -> Result<()> {
    let (subscriber_handle, block_subscriber, tx_manager) =
        instantiate_tx_manager(&config.into()).await?;
    let state_fold = instantiate_state_fold(&config.into())?;

    let mut subscription = block_subscriber
        .subscribe()
        .await
        .ok_or(EmptySubscription {}.build())?;

    while let block_res = subscription.recv().await {
        match block_res {
            Ok(block) => {
                let state = state_fold
                    .get_state_for_block(&config.initial_epoch, block.hash)
                    .await
                    .context(StateFoldError {})?;

                react(state, &tx_manager).await;
            }

            Err(e) => return Err(Error::SubscriberReceiveError { source: e }),
        }
    }

    Ok(())
}

async fn react(
    state: BlockState<DescartesV2State>,
    tx_manager: &DescartesTxManager,
) {
    todo!()
}

impl From<&Config> for instantiate_state_fold::Config {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        Self {
            safety_margin: config.safety_margin,
            input_contract_address: config.input_contract_address,
            descartes_contract_address: config.descartes_contract_address,

            provider_http_url: config.provider_http_url,
            genesis_block: config.genesis_block,
            query_limit_error_codes: config.query_limit_error_codes,
            concurrent_events_fetch: config.concurrent_events_fetch,
        }
    }
}

impl From<&Config> for instantiate_tx_manager::Config {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        Self {
            http_endpoint: config.http_endpoint,
            ws_endpoint: config.ws_endpoint,
            max_retries: config.max_retries,
            max_delay: config.max_delay,

            call_timeout: config.call_timeout,
            subscriber_timeout: config.subscriber_timeout,
        }
    }
}

/*
use super::fold::*;
use dispatcher::state_fold::Access;

use ethers::core::types::{Address, U64};
use ethers::providers::{Http, Provider};

use super::error::*;

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
        Provider::<Http>::try_from(url).context(UrlParseError {})?,
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
*/
