use configuration::error as config_error;

use serde::Deserialize;
use snafu::ResultExt;

use ethers::core::types::{Address, U256};

use std::fs;
use std::str::FromStr;

use structopt::StructOpt;

#[derive(StructOpt, Clone)]
pub struct ApplicationCLIConfig {
    #[structopt(flatten)]
    pub indexer_config: IndexerEnvCLIConfig,
}

#[derive(StructOpt, Clone)]
#[structopt(name = "indexer_config", about = "Configuration for indexer")]
pub struct IndexerEnvCLIConfig {
    /// Address of deployed DApp contract
    #[structopt(long, env)]
    pub dapp_contract_address: Option<String>,

    /// File with ddress of deployed DApp contract
    #[structopt(long, env)]
    pub dapp_contract_address_file: Option<String>,

    #[structopt(long, env)]
    pub contract_name: Option<String>,
    #[structopt(long, env)]
    pub indexer_config_path: Option<String>,
    #[structopt(long)]
    pub state_server_endpoint: Option<String>,
    #[structopt(long)]
    pub interval: Option<u64>,
    #[structopt(long)]
    pub initial_epoch: Option<u64>,
    #[structopt(long)]
    pub postgres_endpoint: Option<String>,
    #[structopt(long)]
    pub mm_endpoint: Option<String>,
    #[structopt(long)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct IndexerFileConfig {
    pub dapp_contract_address: Option<String>,
    pub state_server_endpoint: Option<String>,
    pub interval: Option<u64>,
    pub initial_epoch: Option<u64>,
    pub postgres_endpoint: Option<String>,
    pub mm_endpoint: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub indexer_config: IndexerFileConfig,
}

#[derive(Clone, Debug)]
pub struct IndexerConfig {
    pub dapp_contract_address: Address,
    pub state_server_endpoint: String,
    pub initial_epoch: U256,

    pub interval: u64,

    pub mm_endpoint: String,
    pub postgres_endpoint: String,
    pub session_id: String,
}

impl IndexerConfig {
    pub fn initialize() -> config_error::Result<Self> {
        let app_config = ApplicationCLIConfig::from_args();
        let env_cli_config = app_config.indexer_config;

        let file_config: IndexerFileConfig = {
            let c: FileConfig = configuration::config::load_config_file(
                env_cli_config.indexer_config_path,
            )?;
            c.indexer_config
        };

        let dapp_contract_address: Address = if let Some(a) = env_cli_config
            .dapp_contract_address
            .or(file_config.dapp_contract_address)
        {
            Address::from_str(&a).map_err(|e| {
                config_error::FileError {
                    err: format!(
                        "DApp contract address string ill-formed: {}",
                        e
                    ),
                }
                .build()
            })?
        } else {
            let path = env_cli_config
                .dapp_contract_address_file
                .ok_or(snafu::NoneError)
                .context(config_error::FileError {
                    err: "Must specify either dapp_contract_address or dapp_contract_address_file",
                })?;

            let contents = fs::read_to_string(path.clone()).map_err(|e| {
                config_error::FileError {
                    err: format!("Could not read file at path {}: {}", path, e),
                }
                .build()
            })?;

            Address::from_str(&contents.trim().to_string()).map_err(|e| {
                config_error::FileError {
                    err: format!(
                        "DApp contract address string ill-formed: {}",
                        e
                    ),
                }
                .build()
            })?
        };

        let state_server_endpoint: String = env_cli_config
            .state_server_endpoint
            .or(file_config.state_server_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify state server endpoint",
            })?;

        let initial_epoch: U256 = U256::from(
            env_cli_config
                .initial_epoch
                .or(file_config.initial_epoch)
                .ok_or(snafu::NoneError)
                .context(config_error::FileError {
                    err: "Must specifify initial epoch",
                })?,
        );

        let interval: u64 = env_cli_config
            .interval
            .or(file_config.interval)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify interval",
            })?;

        let mm_endpoint: String = env_cli_config
            .mm_endpoint
            .or(file_config.mm_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify machine manager endpoint",
            })?;

        let postgres_endpoint: String = env_cli_config
            .postgres_endpoint
            .or(file_config.postgres_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify postgres endpoint",
            })?;

        let session_id: String = env_cli_config
            .session_id
            .or(file_config.session_id)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify session id endpoint",
            })?;

        Ok(IndexerConfig {
            dapp_contract_address,
            state_server_endpoint,
            initial_epoch,
            interval,
            mm_endpoint,
            postgres_endpoint,
            session_id,
        })
    }
}
