use configuration;
use configuration::error as config_error;

use serde::Deserialize;
use snafu::ResultExt;

use std::str::FromStr;

use ethers::core::types::{Address, U256};

use structopt::StructOpt;

#[derive(StructOpt, Clone)]
pub struct ApplicationCLIConfig {
    #[structopt(flatten)]
    pub polling_config: PollingEnvCLIConfig,
}

#[derive(StructOpt, Clone)]
#[structopt(name = "polling_config", about = "Configuration for polling")]
pub struct PollingEnvCLIConfig {
    #[structopt(long, env)]
    pub rollups_contract_address: Option<String>,
    #[structopt(long, env)]
    pub polling_config_path: Option<String>,
    #[structopt(long)]
    pub state_server_endpoint: Option<String>,
    #[structopt(long)]
    pub interval: Option<u64>,
    #[structopt(long)]
    pub initial_epoch: Option<u64>,
    #[structopt(long)]
    pub postgres_endpoint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PollingFileConfig {
    pub rollups_contract_address: Option<String>,
    pub state_server_endpoint: Option<String>,
    pub interval: Option<u64>,
    pub initial_epoch: Option<u64>,
    pub postgres_endpoint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub polling_config: PollingFileConfig,
}

#[derive(Clone, Debug)]
pub struct PollingConfig {
    pub rollups_contract_address: Address,
    pub state_server_endpoint: String,
    pub initial_epoch: U256,

    pub interval: u64,
    pub postgres_endpoint: String,
}

impl PollingConfig {
    pub fn initialize() -> config_error::Result<Self> {
        let app_config = ApplicationCLIConfig::from_args();
        let env_cli_config = app_config.polling_config;

        let file_config: PollingFileConfig = {
            let c: FileConfig = configuration::config::load_config_file(
                env_cli_config.polling_config_path,
            )?;
            c.polling_config
        };

        let rollups_contract_address: Address = Address::from_str(
            &env_cli_config
                .rollups_contract_address
                .or(file_config.rollups_contract_address)
                .ok_or(snafu::NoneError)
                .context(config_error::FileError {
                    err: "Must specify rollups contract address",
                })?,
        )
        .map_err(|e| {
            config_error::FileError {
                err: format!(
                    "Rollups contract address string ill-formed: {}",
                    e
                ),
            }
            .build()
        })?;
        
        let state_server_endpoint: String = env_cli_config
            .state_server_endpoint
            .or(file_config.state_server_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify state server endpoint",
            })?;

        let initial_epoch: U256 = U256::from(env_cli_config
            .initial_epoch
            .or(file_config.initial_epoch)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify initial epoch",
            })?);

        let interval: u64 = env_cli_config
            .interval
            .or(file_config.interval)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify interval",
            })?;

        let postgres_endpoint: String = env_cli_config
            .postgres_endpoint
            .or(file_config.postgres_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specifify postgres endpoint",
            })?;


        Ok(PollingConfig {
            rollups_contract_address,
            state_server_endpoint,
            interval,
            initial_epoch,
            postgres_endpoint,
        })
    }
}
