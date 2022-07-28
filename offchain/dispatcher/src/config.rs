use crate::machine::config::{Error as MMError, MMConfig, MMEnvCLIConfig};
use state_client_lib::config::{Error as SCError, SCConfig, SCEnvCLIConfig};
use tx_manager::config::{Error as TxError, TxEnvCLIConfig, TxManagerConfig};

use state_fold_types::ethers::types::{Address, U256};

use snafu::{ResultExt, Snafu};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(name = "rd_config", about = "Configuration for rollups dispatcher")]
pub struct DispatcherEnvCLIConfig {
    #[structopt(flatten)]
    pub sc_config: SCEnvCLIConfig,

    #[structopt(flatten)]
    pub tx_config: TxEnvCLIConfig,

    #[structopt(flatten)]
    pub mm_config: MMEnvCLIConfig,

    /// Address of rollups dapp
    #[structopt(long, env)]
    pub rd_dapp_contract_address: Option<String>,

    /// Path to file with address of rollups dapp
    #[structopt(long, env)]
    pub rd_dapp_contract_address_file: Option<String>,

    /// First epoch that dispatcher will look at. Default zero.
    #[structopt(long, env)]
    pub rd_initial_epoch: Option<U256>,

    /// Minimum required fee for making claims. A value of zero means an
    /// altruistic validator; the node will always make claims regardless
    /// of fee.
    #[structopt(long, env)]
    pub rd_minimum_required_fee: Option<U256>,

    /// Number of future claim fees that the fee manager should have
    /// uncommitted.
    #[structopt(long, env)]
    pub rd_num_buffer_epochs: Option<usize>,

    /// Number of claims before validator redeems fee.
    #[structopt(long, env)]
    pub rd_num_claims_trigger_redeem: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct DispatcherConfig {
    pub sc_config: SCConfig,
    pub tx_config: TxManagerConfig,
    pub mm_config: MMConfig,

    pub dapp_contract_address: Address,
    pub initial_epoch: U256,

    pub minimum_required_fee: U256,
    pub num_buffer_epochs: usize,
    pub num_claims_trigger_redeem: usize,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("StateClient configuration error: {}", source))]
    StateClientError { source: SCError },

    #[snafu(display("TxManager configuration error: {}", source))]
    TxManagerError { source: TxError },

    #[snafu(display("MachineManager configuration error: {}", source))]
    MachineManagerError { source: MMError },

    #[snafu(display("Configuration missing dapp address"))]
    MissingDappAddress {},

    #[snafu(display("Dapp address string parse error"))]
    DappAddressParseError { source: rustc_hex::FromHexError },

    #[snafu(display("Dapp address read file error"))]
    DappAddressReadFileError { source: std::io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_MINIMUM_REQUIRED_FEE: U256 = U256::zero(); // altruistic
const DEFAULT_NUM_BUFFER_EPOCHS: usize = 4;
const DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM: usize = 4;
const DEFAULT_INITIAL_EPOCH: U256 = U256::zero();

impl DispatcherConfig {
    pub fn initialize_from_args() -> Result<Self> {
        let env_cli_config = DispatcherEnvCLIConfig::from_args();
        Self::initialize(env_cli_config)
    }

    pub fn initialize(env_cli_config: DispatcherEnvCLIConfig) -> Result<Self> {
        let sc_config = SCConfig::initialize(env_cli_config.sc_config)
            .context(StateClientSnafu)?;

        let tx_config = TxManagerConfig::initialize(env_cli_config.tx_config)
            .context(TxManagerSnafu)?;

        let mm_config = MMConfig::initialize(env_cli_config.mm_config)
            .context(MachineManagerSnafu)?;

        let dapp_contract_address =
            if let Some(a) = env_cli_config.rd_dapp_contract_address {
                Address::from_str(&a).context(DappAddressParseSnafu)?
            } else {
                let path = env_cli_config
                    .rd_dapp_contract_address_file
                    .ok_or(snafu::NoneError)
                    .context(MissingDappAddressSnafu)?;

                let contents = std::fs::read_to_string(path.clone())
                    .context(DappAddressReadFileSnafu)?;

                Address::from_str(&contents.trim().to_string())
                    .context(DappAddressParseSnafu)?
            };

        let initial_epoch = env_cli_config
            .rd_initial_epoch
            .unwrap_or(DEFAULT_INITIAL_EPOCH);

        let minimum_required_fee = env_cli_config
            .rd_minimum_required_fee
            .unwrap_or(DEFAULT_MINIMUM_REQUIRED_FEE);

        let num_buffer_epochs = env_cli_config
            .rd_num_buffer_epochs
            .unwrap_or(DEFAULT_NUM_BUFFER_EPOCHS);

        let num_claims_trigger_redeem: usize = env_cli_config
            .rd_num_claims_trigger_redeem
            .unwrap_or(DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM);

        assert!(
            sc_config.default_confirmations < tx_config.default_confirmations,
            "`state-client confirmations` has to be less than `tx-manager confirmations,`"
        );

        Ok(DispatcherConfig {
            sc_config,
            tx_config,
            mm_config,

            dapp_contract_address,
            initial_epoch,

            minimum_required_fee,
            num_buffer_epochs,
            num_claims_trigger_redeem,
        })
    }
}
