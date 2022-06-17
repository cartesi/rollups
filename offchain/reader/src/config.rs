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

use reader::error;
/// Reader configuration. Command line parameters take precedence over environment variables
use structopt::StructOpt;
use tracing::error;
use tracing::log::warn;

/// Reader configuration generated from command
/// line arguments. Where both cli and environment arguments are possible
/// although stuctopt generates them automatically they are explicitelly defined for clarity
#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "reader_config", about = "Configuration for reader")]
pub struct ReaderCliConfig {
    /// Postgres database parameters
    /// ****************************
    #[structopt(long = "--postgres-user", env = "POSTGRES_USER")]
    pub postgres_user: String,
    #[structopt(long = "--postgres-password", env = "POSTGRES_PASSWORD")]
    pub postgres_password: Option<String>,
    #[structopt(
        long = "--postgres-password-file",
        env = "POSTGRES_PASSWORD_FILE"
    )]
    pub postgres_password_file: Option<String>,
    #[structopt(
        long = "--postgres-hostname",
        env = "POSTGRES_HOSTNAME",
        default_value = "127.0.0.1"
    )]
    pub postgres_hostname: String,
    #[structopt(
        long = "--postgres-port",
        env = "POSTGRES_PORT",
        default_value = "5432"
    )]
    pub postgres_port: String,
    #[structopt(
        long = "--postgres-db",
        env = "POSTGRES_DB",
        default_value = "postgres"
    )]
    pub postgres_db: String,
    #[structopt(
        long = "--graphql-host",
        env = "GRAPHQL_HOST",
        default_value = "127.0.0.1"
    )]
    pub graphql_host: String,
    #[structopt(
        long = "--graphql-port",
        env = "GRAPHQL_PORT",
        default_value = "4000"
    )]
    pub graphql_port: String,
}

/// Final reader configuration
/// derived from various input configuration options
pub struct ReaderConfig {
    pub postgres_hostname: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub graphql_host: String,
    pub graphql_port: u16,
}

impl ReaderConfig {
    /// Generate reader config from command line arguments and
    /// and environment variables. Mix all parameters taking
    /// into account precedence to form final ReaderConfig
    pub fn initialize() -> error::Result<Self> {
        let reader_cli_config = ReaderCliConfig::from_args();

        let password: String = if let Some(password_filename) =
            reader_cli_config.postgres_password_file
        {
            match std::fs::read_to_string(&password_filename) {
                Ok(password) => {
                    // If both postgres_password and postgres_password_file arguments are used
                    // show warning
                    if reader_cli_config.postgres_password.is_some() {
                        warn!(concat!("Both `postgres_password` and `postgres_password_file` arguments are set, ",
                            " using `postgres_password_file`"));
                    }
                    password
                }
                Err(e) => {
                    error!(
                        "Failed to read password from file: {}",
                        e.to_string()
                    );
                    String::new()
                }
            }
        } else {
            // Password not provided in file, must be provided
            // either as command line argument or env variable
            reader_cli_config
                .postgres_password
                .expect("Database postgres password is not provided")
        };

        Ok(ReaderConfig {
            postgres_hostname: reader_cli_config.postgres_hostname.clone(),
            postgres_port: reader_cli_config
                .postgres_port
                .parse::<u16>()
                .expect("valid database port"),
            postgres_user: reader_cli_config.postgres_user.to_string(),
            postgres_password: password,
            postgres_db: reader_cli_config.postgres_db.clone(),
            graphql_host: reader_cli_config.graphql_host.clone(),
            graphql_port: reader_cli_config
                .graphql_port
                .parse::<u16>()
                .expect("valid port"),
        })
    }
}
