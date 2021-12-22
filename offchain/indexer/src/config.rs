use configuration::error as config_error;
use configuration::config::EnvCLIConfig;
use configuration::config::Config;

use serde::Deserialize;
use snafu::ResultExt;

use ethers::core::types::{Address, U256};

use structopt::StructOpt;

#[derive(StructOpt, Clone)]
pub struct ApplicationCLIConfig {
    #[structopt(flatten)]
    pub basic_config: EnvCLIConfig,
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
    #[structopt(long)]
    pub mm_endpoint: Option<String>,
    #[structopt(long)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PollingFileConfig {
    pub rollups_contract_address: Option<String>,
    pub state_server_endpoint: Option<String>,
    pub interval: Option<u64>,
    pub initial_epoch: Option<u64>,
    pub postgres_endpoint: Option<String>,
    pub mm_endpoint: Option<String>,
    pub session_id: Option<String>,
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

    pub mm_endpoint: String,
    pub postgres_endpoint: String,
    pub session_id: String,
}

impl PollingConfig {
    pub fn initialize() -> config_error::Result<Self> {
        let app_config = ApplicationCLIConfig::from_args();
        let env_cli_config = app_config.polling_config;
        let base_cli_config = app_config.basic_config;

        let file_config: PollingFileConfig = {
            let c: FileConfig = configuration::config::load_config_file(
                env_cli_config.polling_config_path,
            )?;
            c.polling_config
        };
        let basic_config = Config::initialize(base_cli_config)?;

        let rollups_contract_address = basic_config.contracts["RollupsImpl"];

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

        Ok(PollingConfig {
            rollups_contract_address,
            state_server_endpoint,
            initial_epoch,
            interval,
            mm_endpoint,
            postgres_endpoint,
            session_id,
        })
    }
}
