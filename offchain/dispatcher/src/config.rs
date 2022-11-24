use crate::machine::config::{BrokerConfig, BrokerEnvCLIConfig};
use state_client_lib::config::{Error as SCError, SCConfig, SCEnvCLIConfig};
use tx_manager::config::{Error as TxError, TxEnvCLIConfig, TxManagerConfig};

use types::deployment_files::{
    dapp_deployment::DappDeployment,
    rollups_deployment::{RollupsDeployment, RollupsDeploymentJson},
};

use crate::http_health::config::{HealthCheckConfig, HealthCheckEnvCLIConfig};

use snafu::{ResultExt, Snafu};
use std::{fs::File, io::BufReader, path::PathBuf};

use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(name = "rd_config", about = "Configuration for rollups dispatcher")]
pub struct DispatcherEnvCLIConfig {
    #[structopt(flatten)]
    pub sc_config: SCEnvCLIConfig,

    #[structopt(flatten)]
    pub tx_config: TxEnvCLIConfig,

    #[structopt(flatten)]
    pub broker_config: BrokerEnvCLIConfig,

    #[structopt(flatten)]
    pub hc_config: HealthCheckEnvCLIConfig,

    /// Path to file with deployment json of dapp
    #[structopt(
        short,
        long,
        default_value = "./dapp_deployment.json",
        parse(from_os_str)
    )]
    pub rd_dapp_deployment_file: PathBuf,

    /// Path to file with deployment json of rollups
    #[structopt(
        short,
        long,
        default_value = "./rollups_deployment.json",
        parse(from_os_str)
    )]
    pub rd_rollups_deployment_file: PathBuf,

    /// Duration of rollups epoch in seconds, for which dispatcher will make claims.
    #[structopt(short, long, env, default_value = "604800")]
    pub rd_epoch_duration: u64,
}

#[derive(Clone, Debug)]
pub struct DispatcherConfig {
    pub sc_config: SCConfig,
    pub tx_config: TxManagerConfig,
    pub broker_config: BrokerConfig,
    pub hc_config: HealthCheckConfig,

    pub dapp_deployment: DappDeployment,
    pub rollups_deployment: RollupsDeployment,
    pub epoch_duration: u64,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("StateClient configuration error: {}", source))]
    StateClientError { source: SCError },

    #[snafu(display("TxManager configuration error: {}", source))]
    TxManagerError { source: TxError },

    #[snafu(display("Dapp json read file error"))]
    DappJsonReadFileError { source: std::io::Error },

    #[snafu(display("Dapp json parse error"))]
    DappJsonParseError { source: serde_json::Error },

    #[snafu(display("Rollups json read file error"))]
    RollupsJsonReadFileError { source: std::io::Error },

    #[snafu(display("Rollups json parse error"))]
    RollupsJsonParseError { source: serde_json::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

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

        let hc_config = HealthCheckConfig::initialize(env_cli_config.hc_config);

        let dapp_deployment: DappDeployment = {
            let path = env_cli_config.rd_dapp_deployment_file;
            let file = File::open(path).context(DappJsonReadFileSnafu)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).context(DappJsonParseSnafu)?
        };

        let rollups_deployment: RollupsDeployment = {
            let path = env_cli_config.rd_rollups_deployment_file;
            let file = File::open(path).context(DappJsonReadFileSnafu)?;
            let reader = BufReader::new(file);
            let deployment: RollupsDeploymentJson =
                serde_json::from_reader(reader).context(DappJsonParseSnafu)?;
            deployment.into()
        };

        let broker_config = BrokerConfig::initialize(
            env_cli_config.broker_config,
            tx_config.chain_id,
            dapp_deployment.dapp_address.clone().to_fixed_bytes(),
        );

        let epoch_duration = env_cli_config.rd_epoch_duration;

        assert!(
            sc_config.default_confirmations < tx_config.default_confirmations,
            "`state-client confirmations` has to be less than `tx-manager confirmations,`"
        );

        Ok(DispatcherConfig {
            sc_config,
            tx_config,
            broker_config,
            hc_config,

            dapp_deployment,
            rollups_deployment,
            epoch_duration,
        })
    }
}
