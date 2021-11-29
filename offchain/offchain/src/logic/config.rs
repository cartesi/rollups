use configuration;
use configuration::error as config_error;

use ethers::core::types::{Address, U256};
use offchain_core::ethers;

use serde::Deserialize;
use snafu::ResultExt;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "logic_config", about = "Configuration for rollups logic")]
pub struct LogicEnvCLIConfig {
    /// Path to logic .toml config
    #[structopt(long, env)]
    pub logic_config_path: Option<String>,
    /// Signer Mnemonic
    #[structopt(long, env = "MNEMONIC")]
    pub mnemonic: Option<String>,
    /// Address of deployed rollups contract
    #[structopt(long, env)]
    pub rollups_contract_address: Option<String>,
    /// URL of provider http endpoint
    #[structopt(long, env)]
    pub provider_http_endpoint: Option<String>,
    /// URL of websocket provider endpoint
    #[structopt(long, env)]
    pub ws_endpoint: Option<String>,
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
    pub mnemonic: Option<String>,
    pub rollups_contract_address: Option<String>,
    pub provider_http_endpoint: Option<String>,
    pub ws_endpoint: Option<String>,
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
    pub mnemonic: String,
    pub rollups_contract_address: Address,
    pub initial_epoch: U256,

    pub provider_http_endpoint: String,
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
const DEFAULT_INITIAL_EPOCH: u64 = 0;

const DEFAULT_PROVIDER_HTTP_ENDPOINT: &str = "http://localhost:8545";
const DEFAULT_PROVIDER_WS_ENDPOINT: &str = "ws://localhost:8546";
const DEFAULT_FOLD_SERVER_ENPOINT: &str = "http://localhost:50051";

const DEFAULT_MACHINE_MANAGER_ENPOINT: &str = "http://localhost:8000";
const DEFAULT_SESSION_ID: &str = "DEFAULT_SESSION_ID";

const DEFAULT_RATE: usize = 20;
const DEFAULT_CONFIRMATIONS: usize = 10;

impl LogicConfig {
    pub fn initialize(
        env_cli_config: LogicEnvCLIConfig,
    ) -> config_error::Result<Self> {
        let file_config: LogicFileConfig = {
            let c: FileConfig = configuration::config::load_config_file(
                env_cli_config.logic_config_path,
            )?;
            c.logic_config
        };

        let mnemonic: String = env_cli_config
            .mnemonic
            .or(file_config.mnemonic)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify mnemonic",
            })?;

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

        let initial_epoch: U256 = U256::from(
            env_cli_config
                .initial_epoch
                .or(file_config.initial_epoch)
                .unwrap_or(DEFAULT_INITIAL_EPOCH),
        );

        let provider_http_endpoint: String = env_cli_config
            .provider_http_endpoint
            .or(file_config.provider_http_endpoint)
            .unwrap_or(DEFAULT_PROVIDER_HTTP_ENDPOINT.to_owned());

        let ws_endpoint: String = env_cli_config
            .ws_endpoint
            .or(file_config.ws_endpoint)
            .unwrap_or(DEFAULT_PROVIDER_WS_ENDPOINT.to_owned());

        let state_fold_grpc_endpoint: String = env_cli_config
            .state_fold_grpc_endpoint
            .or(file_config.state_fold_grpc_endpoint)
            .unwrap_or(DEFAULT_FOLD_SERVER_ENPOINT.to_owned());

        let mm_endpoint: String = env_cli_config
            .mm_endpoint
            .or(file_config.mm_endpoint)
            .unwrap_or(DEFAULT_MACHINE_MANAGER_ENPOINT.to_owned());

        let session_id: String = env_cli_config
            .session_id
            .or(file_config.session_id)
            .unwrap_or(DEFAULT_SESSION_ID.to_owned());

        let gas_multiplier: Option<f64> =
            env_cli_config.gas_multiplier.or(file_config.gas_multiplier);

        let gas_price_multiplier: Option<f64> = env_cli_config
            .gas_price_multiplier
            .or(file_config.gas_price_multiplier);

        let rate: usize = env_cli_config
            .rate
            .or(file_config.rate)
            .unwrap_or(DEFAULT_RATE);

        let confirmations: usize = env_cli_config
            .confirmations
            .or(file_config.confirmations)
            .unwrap_or(DEFAULT_CONFIRMATIONS);

        Ok(LogicConfig {
            mnemonic,
            rollups_contract_address,
            initial_epoch,

            provider_http_endpoint,
            ws_endpoint,
            state_fold_grpc_endpoint,

            mm_endpoint,
            session_id,

            gas_multiplier,
            gas_price_multiplier,
            rate,
            confirmations,
        })
    }
}
