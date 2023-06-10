// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// Configuration can be provided using command-line options, environment variables or
/// configuration file.
/// Command-line parameters take precedence over environment variables and environment variables
/// take precedence over same parameter from file configuration.
use serde::Deserialize;
use snafu::{whatever, OptionExt, ResultExt, Snafu};
use structopt::StructOpt;

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("parse configuration file error"))]
    FileError { source: std::io::Error },

    #[snafu(display("parse configuration file error"))]
    ParseError { source: toml::de::Error },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

/// Final config structure exported by this module
#[derive(Clone, Debug)]
pub struct Config {
    pub inspect_server_address: String,
    pub server_manager_address: String,
    pub session_id: String,
    pub queue_size: usize,
    pub inspect_path_prefix: String,
    pub healthcheck_path: String,
}

impl Config {
    /// Generate config from command line arguments and and environment variables.
    /// If config path is provided, read configuration from the file.
    pub fn initialize() -> Result<Self, ConfigError> {
        let env_cli_config = EnvCLIConfig::from_args();
        let file_config: FileConfig =
            load_config_file(env_cli_config.config_path)?;

        let inspect_server_address: String = env_cli_config
            .inspect_server_address
            .or(file_config.inspect_server_address)
            .whatever_context(
                "must specify inspect server address".to_string(),
            )?;

        let server_manager_address: String = env_cli_config
            .server_manager_address
            .or(file_config.server_manager_address)
            .whatever_context(
                "must specify server manager address".to_string(),
            )?;

        let session_id: String = env_cli_config
            .session_id
            .or(file_config.session_id)
            .whatever_context("must specify session id".to_string())?;

        let queue_size: usize = env_cli_config
            .queue_size
            .or(file_config.queue_size)
            .unwrap_or(100);

        let healthcheck_path: String = env_cli_config
            .healthcheck_path
            .or(file_config.healthcheck_path)
            .map(check_path_prefix)
            .unwrap_or(Ok(String::from("/healthz")))?;

        let inspect_path_prefix: String = env_cli_config
            .inspect_path_prefix
            .or(file_config.inspect_path_prefix)
            .map(check_path_prefix)
            .unwrap_or(Ok(String::from("/inspect")))?;
        if inspect_path_prefix == healthcheck_path {
            whatever!("inspect path must be different from healthcheck path");
        }

        Ok(Self {
            inspect_server_address,
            server_manager_address,
            session_id,
            queue_size,
            inspect_path_prefix,
            healthcheck_path,
        })
    }
}

#[derive(StructOpt, Clone)]
#[structopt(name = "inspect-server")]
struct EnvCLIConfig {
    /// HTTP address for the inspect server
    #[structopt(long, env)]
    inspect_server_address: Option<String>,

    /// Server manager gRPC address
    #[structopt(long, env)]
    server_manager_address: Option<String>,

    /// Server manager session id
    #[structopt(long, env)]
    session_id: Option<String>,

    /// Path prefix for the inspect server URL
    #[structopt(long, env)]
    inspect_path_prefix: Option<String>,

    /// Path for the healthcheck check
    #[structopt(long, env)]
    healthcheck_path: Option<String>,

    /// Queue size for concurrent inspect requests
    #[structopt(long, env)]
    queue_size: Option<usize>,

    /// Path to the config file
    #[structopt(long, env)]
    pub config_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct FileConfig {
    inspect_server_address: Option<String>,
    server_manager_address: Option<String>,
    session_id: Option<String>,
    queue_size: Option<usize>,
    inspect_path_prefix: Option<String>,
    healthcheck_path: Option<String>,
}

fn check_path_prefix(prefix: String) -> Result<String, ConfigError> {
    let re = regex::Regex::new(r"^/[a-z]+$").unwrap();
    if re.is_match(&prefix) {
        Ok(prefix)
    } else {
        whatever!("invalid path prefix, it should be in the format `/[a-z]+`.");
    }
}

fn load_config_file<T: Default + serde::de::DeserializeOwned>(
    // path to the config file if provided
    config_file: Option<String>,
) -> Result<T, ConfigError> {
    match config_file {
        Some(config) => {
            let s = std::fs::read_to_string(&config).context(FileSnafu)?;

            let file_config: T = toml::from_str(&s).context(ParseSnafu)?;

            Ok(file_config)
        }
        None => Ok(T::default()),
    }
}
