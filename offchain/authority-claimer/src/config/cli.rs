// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use clap::{command, Parser};
use eth_tx_manager::{
    config::{TxEnvCLIConfig as TxManagerCLIConfig, TxManagerConfig},
    Priority,
};
use rollups_events::{BrokerCLIConfig, BrokerConfig};
use rusoto_core::Region;
use snafu::ResultExt;
use std::{fs, path::PathBuf, str::FromStr};

use crate::config::{
    error::{
        AuthConfigError, AuthSnafu, AuthorityClaimerConfigError,
        InvalidRegionSnafu, MnemonicFileSnafu, TxManagerSnafu,
    },
    json::{read_json_file, DappDeployment},
    AuthConfig, AuthorityClaimerConfig,
};

// ------------------------------------------------------------------------------------------------
// AuthorityClaimerCLI
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Parser)]
#[command(name = "rd_config")]
#[command(about = "Configuration for rollups authority claimer")]
pub(crate) struct AuthorityClaimerCLI {
    #[command(flatten)]
    txm_config: TxManagerCLIConfig,

    #[command(flatten)]
    auth_config: AuthCLIConfig,

    #[command(flatten)]
    broker_config: BrokerCLIConfig,

    /// Path to file with deployment json of dapp
    #[arg(long, env, default_value = "./dapp_deployment.json")]
    dapp_deployment_file: PathBuf,
}

impl TryFrom<AuthorityClaimerCLI> for AuthorityClaimerConfig {
    type Error = AuthorityClaimerConfigError;

    fn try_from(cli_config: AuthorityClaimerCLI) -> Result<Self, Self::Error> {
        let txm_config = TxManagerConfig::initialize(cli_config.txm_config)
            .context(TxManagerSnafu)?;

        let auth_config =
            AuthConfig::try_from(cli_config.auth_config).context(AuthSnafu)?;

        let broker_config = BrokerConfig::from(cli_config.broker_config);

        let dapp_deployment =
            read_json_file::<DappDeployment>(cli_config.dapp_deployment_file)?;
        let dapp_address = dapp_deployment.dapp_address;
        let dapp_deploy_block_hash = dapp_deployment.dapp_deploy_block_hash;

        Ok(AuthorityClaimerConfig {
            txm_config,
            auth_config,
            broker_config,
            dapp_address,
            dapp_deploy_block_hash,
            txm_priority: Priority::Normal,
        })
    }
}

// ------------------------------------------------------------------------------------------------
// AuthConfig
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Parser)]
#[command(name = "auth_config")]
#[command(about = "Configuration for signing authentication")]
pub(crate) struct AuthCLIConfig {
    /// Signer mnemonic, overrides `auth_mnemonic_file` and `auth_aws_kms_*`
    #[arg(long, env)]
    auth_mnemonic: Option<String>,

    /// Signer mnemonic file path, overrides `auth_aws_kms_*`
    #[arg(long, env)]
    auth_mnemonic_file: Option<String>,

    /// Mnemonic account index
    #[arg(long, env)]
    auth_mnemonic_account_index: Option<u32>,

    /// AWS KMS signer key-id
    #[arg(long, env)]
    auth_aws_kms_key_id: Option<String>,

    /// AWS KMS signer region
    #[arg(long, env)]
    auth_aws_kms_region: Option<String>,
}

impl TryFrom<AuthCLIConfig> for AuthConfig {
    type Error = AuthConfigError;

    fn try_from(cli: AuthCLIConfig) -> Result<Self, Self::Error> {
        let account_index = cli.auth_mnemonic_account_index;
        if let Some(mnemonic) = cli.auth_mnemonic {
            Ok(AuthConfig::Mnemonic {
                mnemonic,
                account_index,
            })
        } else if let Some(path) = cli.auth_mnemonic_file {
            let mnemonic = fs::read_to_string(path.clone())
                .context(MnemonicFileSnafu { path })?
                .trim()
                .to_string();
            Ok(AuthConfig::Mnemonic {
                mnemonic,
                account_index,
            })
        } else {
            match (cli.auth_aws_kms_key_id, cli.auth_aws_kms_region) {
                (None, _) => Err(AuthConfigError::MissingConfiguration),
                (Some(_), None) => Err(AuthConfigError::MissingRegion),
                (Some(key_id), Some(region)) => {
                    let region = Region::from_str(&region)
                        .context(InvalidRegionSnafu)?;
                    Ok(AuthConfig::Aws { key_id, region })
                }
            }
        }
    }
}
