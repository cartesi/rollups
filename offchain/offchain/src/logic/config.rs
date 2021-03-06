use configuration;
use configuration::error as config_error;

use ethers::core::types::{Address, U256};
use offchain_core::ethers;

use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "logic_config", about = "Configuration for rollups logic")]
pub struct LogicEnvCLIConfig {
    /// Path to logic .toml config
    #[structopt(long, env)]
    pub logic_config_path: Option<String>,
    /// Signer mnemonic
    #[structopt(long, env = "MNEMONIC")]
    pub mnemonic: Option<String>,
    /// Signer mnemonic file path
    #[structopt(long, env = "MNEMONIC_FILE")]
    pub mnemonic_file: Option<String>,
    /// Chain ID
    #[structopt(long, env)]
    pub chain_id: Option<u64>,
    /// Address of deployed DApp contract
    #[structopt(long, env)]
    pub dapp_contract_address: Option<String>,
    /// File with ddress of deployed DApp contract
    #[structopt(long, env)]
    pub dapp_contract_address_file: Option<String>,
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
    /// Minimum required fee for making claims. A value of zero means an
    /// altruistic validator; the node will always make claims regardless
    /// of fee.
    #[structopt(long, env)]
    pub minimum_required_fee: Option<String>,
    /// Number of future claim fees that the fee manager should have
    /// uncommitted.
    #[structopt(long, env)]
    pub num_buffer_epochs: Option<usize>,
    /// Number of claims before validator redeems fee.
    #[structopt(long, env)]
    pub num_claims_trigger_redeem: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct LogicFileConfig {
    pub mnemonic: Option<String>,
    pub mnemonic_file: Option<String>,
    pub chain_id: Option<u64>,
    pub dapp_contract_address: Option<String>,
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
    pub minimum_required_fee: Option<String>,
    pub num_buffer_epochs: Option<usize>,
    pub num_claims_trigger_redeem: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub logic_config: LogicFileConfig,
}

#[derive(Clone, Debug)]
pub struct LogicConfig {
    pub mnemonic: String,
    pub chain_id: u64,
    pub dapp_contract_address: Address,
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

    pub minimum_required_fee: U256,
    pub num_buffer_epochs: usize,
    pub num_claims_trigger_redeem: usize,
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

const DEFAULT_NUM_BUFFER_EPOCHS: usize = 4;
const DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM: usize = 4;

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

        let mnemonic: String = if let Some(m) =
            env_cli_config.mnemonic.or(file_config.mnemonic)
        {
            m
        } else {
            let path = env_cli_config
                .mnemonic_file
                .or(file_config.mnemonic_file)
                .ok_or(snafu::NoneError)
                .context(config_error::FileError {
                    err: "Must specify either mnemonic or mnemonic_file",
                })?;

            let contents = fs::read_to_string(path.clone()).map_err(|e| {
                config_error::FileError {
                    err: format!("Could not read file at path {}: {}", path, e),
                }
                .build()
            })?;

            contents.trim().to_string()
        };

        let chain_id = env_cli_config
            .chain_id
            .or(file_config.chain_id)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify chain_id",
            })?;

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

        let minimum_required_fee: U256 = match env_cli_config
            .minimum_required_fee
            .or(file_config.minimum_required_fee)
        {
            Some(s) => U256::from_dec_str(&s).map_err(|e| {
                config_error::FileError {
                    err: format!("Minimum fee string ill-formed: {}", e),
                }
                .build()
            })?,

            None => U256::zero(),
        };

        let num_buffer_epochs: usize = env_cli_config
            .num_buffer_epochs
            .or(file_config.num_buffer_epochs)
            .unwrap_or(DEFAULT_NUM_BUFFER_EPOCHS);

        let num_claims_trigger_redeem: usize = env_cli_config
            .num_claims_trigger_redeem
            .or(file_config.num_claims_trigger_redeem)
            .unwrap_or(DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM);

        Ok(LogicConfig {
            mnemonic,
            chain_id,
            dapp_contract_address,
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

            minimum_required_fee,
            num_buffer_epochs,
            num_claims_trigger_redeem,
        })
    }
}
