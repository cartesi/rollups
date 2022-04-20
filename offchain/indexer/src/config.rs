/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

/// Configuration for the indexer
/// Configuration to the indexer can be provided using command line options, environment variables
/// or configuration file. In general, command line indexer parameters take precedence over environment variables
/// and environment variables take precedence over same parameter from file configuration
use configuration::config::Config;
use configuration::config::EnvCLIConfig;
use configuration::error as config_error;

use serde::Deserialize;
use snafu::ResultExt;

use ethers::core::types::{Address, U256};
use tracing::{error, warn};

use structopt::StructOpt;

/// Application configuration generated from
/// command line arguments
#[derive(StructOpt, Clone)]
pub struct ApplicationCLIConfig {
    #[structopt(flatten)]
    pub basic_config: EnvCLIConfig,
    #[structopt(flatten)]
    pub indexer_config: IndexerEnvCLIConfig,
}

/// Indexer configuration generated from command
/// line arguments. Where both cli and environment arguments are possible
/// although stuctopt generates them automatically they are explicitelly defined for clarity
#[derive(StructOpt, Clone)]
#[structopt(name = "indexer_config", about = "Configuration for indexer")]
pub struct IndexerEnvCLIConfig {
    #[structopt(long, env)]
    pub dapp_contract_address: Option<String>,
    #[structopt(long, env)]
    pub dapp_contract_name: Option<String>,
    #[structopt(long, env)]
    pub indexer_config_path: Option<String>,
    #[structopt(long)]
    pub state_server_endpoint: Option<String>,
    #[structopt(long)]
    pub interval: Option<u64>,
    #[structopt(long)]
    pub initial_epoch: Option<u64>,
    #[structopt(long)]
    pub mm_endpoint: Option<String>,
    #[structopt(long)]
    pub session_id: Option<String>,
    #[structopt(long = "--postgres-user", env = "POSTGRES_USER")]
    pub postgres_user: Option<String>,
    #[structopt(long = "--postgres-password", env = "POSTGRES_PASSWORD")]
    pub postgres_password: Option<String>,
    #[structopt(
        long = "--postgres-password-file",
        env = "POSTGRES_PASSWORD_FILE"
    )]
    pub postgres_password_file: Option<String>,
    #[structopt(long = "--postgres-hostname", env = "POSTGRES_HOSTNAME")]
    pub postgres_hostname: Option<String>,
    #[structopt(long = "--postgres-port", env = "POSTGRES_PORT")]
    pub postgres_port: Option<u16>,
    #[structopt(long = "--postgres-db", env = "POSTGRES_DB")]
    pub postgres_db: Option<String>,
}

/// Indexer configuration deserialized from file
/// (usually indexer-config.toml defined with rollup application)
#[derive(Clone, Debug, Deserialize, Default)]
pub struct IndexerFileConfig {
    pub dapp_contract_address: Option<String>,
    pub state_server_endpoint: Option<String>,
    pub interval: Option<u64>,
    pub initial_epoch: Option<u64>,
    pub mm_endpoint: Option<String>,
    pub session_id: Option<String>,
    pub dapp_contract_name: Option<String>,
    pub postgres_hostname: Option<String>,
    pub postgres_port: Option<u16>,
    pub postgres_user: Option<String>,
    pub postgres_password: Option<String>,
    pub postgres_password_file: Option<String>,
    pub postgres_db: Option<String>,
}

/// Indexer file configuration
#[derive(Clone, Debug, Deserialize, Default)]
pub struct FileConfig {
    pub indexer_config: IndexerFileConfig,
}

/// Final database configuration (needed for database handling)
/// derived from various input configuration options
#[derive(Clone, Debug)]
pub struct PostgresConfig {
    pub postgres_hostname: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
}

/// Final indexer configuration
/// derived from various input configuration options
#[derive(Clone, Debug)]
pub struct IndexerConfig {
    pub dapp_contract_address: Address,
    pub state_server_endpoint: String,
    pub initial_epoch: U256,
    pub interval: u64,
    pub mm_endpoint: String,
    pub session_id: String,
    pub database: PostgresConfig,
}

impl IndexerConfig {
    /// Generate application config from command line arguments and
    /// and environment variables. If  indexer config path is provided
    /// read indexer configuration from the file. Mix all parameters taking
    /// into account precedence to form final IndexerConfig
    pub fn initialize() -> config_error::Result<Self> {
        let app_config = ApplicationCLIConfig::from_args();
        let env_cli_config = app_config.indexer_config;
        let base_cli_config = app_config.basic_config;

        let file_config: IndexerFileConfig = {
            let c: FileConfig = configuration::config::load_config_file(
                env_cli_config.indexer_config_path,
            )?;
            c.indexer_config
        };
        let basic_config = Config::initialize(base_cli_config)?;

        let dapp_contract_name = env_cli_config
            .dapp_contract_name
            .or(file_config.dapp_contract_name)
            .unwrap_or_else(|| "CartesiDApp".to_string());

        let dapp_contract_address = basic_config.contracts[&dapp_contract_name];

        let state_server_endpoint: String = env_cli_config
            .state_server_endpoint
            .or(file_config.state_server_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify state server endpoint",
            })?;

        let initial_epoch: U256 = U256::from(
            env_cli_config
                .initial_epoch
                .or(file_config.initial_epoch)
                .ok_or(snafu::NoneError)
                .context(config_error::FileError {
                    err: "Must specify initial epoch",
                })?,
        );

        // Polling interval
        let interval: u64 = env_cli_config
            .interval
            .or(file_config.interval)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify interval",
            })?;

        // Cartesi server manager endpoint
        let mm_endpoint: String = env_cli_config
            .mm_endpoint
            .or(file_config.mm_endpoint)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify machine manager endpoint",
            })?;

        let session_id: String = env_cli_config
            .session_id
            .or(file_config.session_id)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify session id endpoint",
            })?;

        let postgres_hostname: String = env_cli_config
            .postgres_hostname
            .or(file_config.postgres_hostname)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify postgres hostname",
            })?;

        // We use default postgres port if no other provided
        let postgres_port: u16 = env_cli_config
            .postgres_port
            .or(file_config.postgres_port)
            .unwrap_or(5432);

        let postgres_user: String = env_cli_config
            .postgres_user
            .or(file_config.postgres_user)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify postgres user",
            })?;

        let postgres_password_file: Option<String> = env_cli_config
            .postgres_password_file
            .or(file_config.postgres_password_file);

        // Password can also be read from file, in which case
        // takes the precedence
        let password_from_file: Option<String> = if let Some(
            password_filename,
        ) = postgres_password_file
        {
            match std::fs::read_to_string(&password_filename) {
                Ok(password) => {
                    if env_cli_config.postgres_password.is_some() {
                        warn!(concat!("Both `postgres_password` and `postgres_password_file` arguments are set, ",
                            "using `postgres_password_file`"));
                    } else if file_config.postgres_password.is_some() {
                        warn!(concat!("Both `postgres_password` in config file and `postgres_password_file` ",
                            "arguments are set, using `postgres_password_file`"));
                    }
                    Some(password)
                }
                Err(e) => {
                    error!(
                        "Failed to read password from file: {}",
                        e.to_string()
                    );
                    None
                }
            }
        } else {
            None
        };

        let postgres_password: String = password_from_file
            .or(env_cli_config.postgres_password)
            .or(file_config.postgres_password)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify postgres password",
            })?;

        let postgres_db: String = env_cli_config
            .postgres_db
            .or(file_config.postgres_db)
            .ok_or(snafu::NoneError)
            .context(config_error::FileError {
                err: "Must specify postgres database",
            })?;

        Ok(IndexerConfig {
            dapp_contract_address,
            state_server_endpoint,
            initial_epoch,
            interval,
            mm_endpoint,
            session_id,
            database: PostgresConfig {
                postgres_hostname,
                postgres_port,
                postgres_user,
                postgres_password,
                postgres_db,
            },
        })
    }
}
