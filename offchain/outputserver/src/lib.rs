pub mod config;
pub mod delegate_manager;
pub mod error;

use ethers::core::types::Address;

use snafu::Snafu;

use offchain::logic::instantiate_state_fold::create_access;
use offchain::logic::instantiate_state_fold::{Config, DescartesAccess};

use offchain::fold::setup::{create_output, OutputStateFold, SetupConfig};

use config::ApplicationConfig;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    BadConfiguration { err: String },
}

pub fn initialize_config() -> Config {
    let config = ApplicationConfig::initialize().unwrap();

    Config {
        safety_margin: config.sf_config.safety_margin,
        input_contract_address: Address::zero(), /* Won't be used */
        output_contract_address: Address::zero(), /* Won't be used */
        descartes_contract_address: config.basic_config.contracts
            ["DescartesV2Impl"],
        provider_http_url: config.basic_config.url,
        genesis_block: config.sf_config.genesis_block,
        query_limit_error_codes: config.sf_config.query_limit_error_codes,
        concurrent_events_fetch: config.sf_config.concurrent_events_fetch,
    }
}

pub fn instantiate_output_fold_delegate() -> OutputStateFold<DescartesAccess> {
    let config: Config = initialize_config();
    let access = create_access(&config).unwrap();
    let setup_config = SetupConfig {
        safety_margin: config.safety_margin,
        input_contract_address: config.input_contract_address,
        output_contract_address: config.output_contract_address,
        descartes_contract_address: config.descartes_contract_address,
    };

    create_output(access, &setup_config)
}
