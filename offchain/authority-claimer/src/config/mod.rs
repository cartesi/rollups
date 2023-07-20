// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

mod cli;
mod error;
mod json;

pub use error::{AuthConfigError, AuthorityClaimerConfigError};

use cli::AuthorityClaimerCLI;
use eth_tx_manager::{config::TxManagerConfig, Priority};
use ethers::types::{Address, H256};
use http_server::HttpServerConfig;
use rollups_events::BrokerConfig;
use rusoto_core::Region;

#[derive(Debug, Clone)]
pub struct Config {
    pub authority_claimer_config: AuthorityClaimerConfig,
    pub http_server_config: HttpServerConfig,
}

#[derive(Debug, Clone)]
pub struct AuthorityClaimerConfig {
    pub txm_config: TxManagerConfig,
    pub auth_config: AuthConfig,
    pub broker_config: BrokerConfig,
    pub dapp_address: Address, // TODO: can I use rollups_events types?
    pub dapp_deploy_block_hash: H256, // TODO: can I use rollups_events types?
    pub txm_priority: Priority,
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Mnemonic {
        mnemonic: String,
        account_index: Option<u32>,
    },

    Aws {
        key_id: String,
        region: Region,
    },
}

impl Config {
    pub fn new() -> Result<Self, AuthorityClaimerConfigError> {
        let (http_server_config, authority_claimer_cli) =
            HttpServerConfig::parse::<AuthorityClaimerCLI>("authority_claimer");
        let authority_claimer_config = authority_claimer_cli.try_into()?;
        Ok(Self {
            authority_claimer_config,
            http_server_config,
        })
    }
}
