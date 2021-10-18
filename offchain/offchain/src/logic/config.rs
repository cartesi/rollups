use configuration;
use configuration::error as config_error;

use ethers::core::types::{Address, U256};
use offchain_core::ethers;

use serde::Deserialize;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "logic_config", about = "Configuration for rollups logic")]
pub struct LogicEnvCLIConfig {
    /// Path to logic .toml config
    #[structopt(long, env)]
    pub logic_config_path: Option<String>,
    /// Signer address
    #[structopt(long, env)]
    pub sender: Option<String>,
    /// Address of deployed descartes contract
    #[structopt(long, env)]
    pub descartes_contract_address: Option<String>,
    /// URL of transaction signer http endpoint
    #[structopt(long, env)]
    pub signer_http_endpoint: Option<String>,
    /// URL of state fold server gRPC endpoint
    #[structopt(long, env)]
    pub state_fold_grpc_endpoint: Option<String>,
    /// Initial epoch of state fold indexing
    #[structopt(long, env)]
    pub initial_epoch: Option<u64>,
    /// Tx gas multiplier
    #[structopt(long, env)]
    pub gas_multiplier: Option<f64>,
    /// Tx gas price multiplier
    #[structopt(long, env)]
    pub gas_price_multiplier: Option<f64>,
    /// Tx resubmit rate (for Tx Manager)
    #[structopt(long, env)]
    pub rate: Option<usize>,
    /// Tx confirmations (for Tx Manager)
    #[structopt(long, env)]
    pub confirmations: Option<usize>,
    /// URL of rollups machine manager gRPC endpoint
    #[structopt(long, env)]
    pub mm_endpoint: Option<String>,
    /// Session ID for rollups machine manager
    #[structopt(long, env)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct LogicFileConfig {
    pub sender: Option<String>,
    pub descartes_contract_address: Option<String>,
    pub signer_http_endpoint: Option<String>,
    pub state_fold_grpc_endpoint: Option<String>,
    pub initial_epoch: Option<u64>,
    pub gas_multiplier: Option<f64>,
    pub gas_price_multiplier: Option<f64>,
    pub rate: Option<usize>,
    pub confirmations: Option<usize>,
    pub mm_endpoint: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub logic_config: LogicFileConfig,
}

#[derive(Clone, Debug)]
pub struct LogicConfig {
    pub sender: Address,
    pub descartes_contract_address: Address,
    pub initial_epoch: U256,

    pub signer_http_endpoint: String,
    pub ws_endpoint: String,
    pub state_fold_grpc_endpoint: String,

    pub mm_endpoint: String,
    pub session_id: String,

    pub gas_multiplier: Option<f64>,
    pub gas_price_multiplier: Option<f64>,
    pub rate: usize,
    pub confirmations: usize,
}

// default values
const DEFAULT_INITIAL_EPOCH: U256 = U256::zero();

const DEFAULT_PROVIDER_HTTP_ENDPOINT: &str = "http://localhost:8545";
const DEFAULT_PROVIDER_WS_ENDPOINT: &str = "ws://localhost:8546";
const DEFAULT_FOLD_SERVER_ENPOINT: &str = "http://localhost:50051";

const DEFAULT_MACHINE_MANAGER_ENPOINT: &str = "http://localhost:8000";
const DEFAULT_SESSION_ID: &str = "DEFAULT_SESSION_ID";

// const DEFAULT_GAS_MULTIPLIER: f64 = 1.0;
// const DEFAULT_GAS_PRICE_MULTIPLIER: f64 = 1.0;
const DEFAULT_RATE: usize = 20;
const DEFAULT_CONFIRMATIONS: usize = 10;

impl LogicConfig {
    pub fn initialize(
        env_cli_config: LogicEnvCLIConfig,
    ) -> config_error::Result<Self> {
        let file_config: FileConfig = configuration::config::load_config_file(
            env_cli_config.logic_config_path,
        )?;

        let max_delay = Duration::from_secs(
            env_cli_config
                .tm_max_delay
                .or(file_config.tx_manager.max_delay)
                .unwrap_or(DEFAULT_MAX_DELAY),
        );

        let max_retries = env_cli_config
            .tm_max_retries
            .or(file_config.tx_manager.max_retries)
            .unwrap_or(DEFAULT_MAX_RETRIES);

        let transaction_timeout = Duration::from_secs(
            env_cli_config
                .tm_timeout
                .or(file_config.tx_manager.timeout)
                .unwrap_or(DEFAULT_TIMEOUT),
        );

        Ok(TMConfig {
            max_delay,
            max_retries,
            transaction_timeout,
        })
    }
}

/*
use configuration::error as config_error;

use serde::Deserialize;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(
    name = "tm_config",
    about = "Configuration for transaction manager"
)]
pub struct TMEnvCLIConfig {
    /// Path to transaction manager .toml config
    #[structopt(long, env)]
    pub tm_config: Option<String>,
    /// Max delay (secs) between retries
    #[structopt(long, env)]
    pub tm_max_delay: Option<u64>,
    /// Max retries for a transaction
    #[structopt(long, env)]
    pub tm_max_retries: Option<usize>,
    /// Timeout value (secs) for a transaction
    #[structopt(long, env)]
    pub tm_timeout: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct TMFileConfig {
    pub max_delay: Option<u64>,
    pub max_retries: Option<usize>,
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub tx_manager: TMFileConfig,
}

#[derive(Clone, Debug)]
pub struct TMConfig {
    pub max_delay: Duration,
    pub max_retries: usize,
    pub transaction_timeout: Duration,
}

// default values
const DEFAULT_MAX_DELAY: u64 = 1;
const DEFAULT_MAX_RETRIES: usize = 5;
const DEFAULT_TIMEOUT: u64 = 5;

impl TMConfig {
    pub fn initialize(
        env_cli_config: TMEnvCLIConfig,
    ) -> config_error::Result<Self> {
        let file_config: FileConfig =
            configuration::config::load_config_file(env_cli_config.tm_config)?;

        let max_delay = Duration::from_secs(
            env_cli_config
                .tm_max_delay
                .or(file_config.tx_manager.max_delay)
                .unwrap_or(DEFAULT_MAX_DELAY),
        );

        let max_retries = env_cli_config
            .tm_max_retries
            .or(file_config.tx_manager.max_retries)
            .unwrap_or(DEFAULT_MAX_RETRIES);

        let transaction_timeout = Duration::from_secs(
            env_cli_config
                .tm_timeout
                .or(file_config.tx_manager.timeout)
                .unwrap_or(DEFAULT_TIMEOUT),
        );

        Ok(TMConfig {
            max_delay,
            max_retries,
            transaction_timeout,
        })
    }
}
*/
