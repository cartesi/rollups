// Copyright 2022 Cartesi Pte. Ltd.
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
use snafu::Snafu;
use structopt::StructOpt;

#[derive(Debug, Snafu)]
#[snafu(display("Bad configuration: {}", err))]
pub struct BadConfigurationError {
    err: String,
}

/// Final config structure exported by this module
#[derive(Clone, Debug)]
pub struct Config {
    pub inspect_server_address: String,
    pub server_manager_address: String,
    pub session_id: String,
    pub inspect_path_prefix: String,
    pub healthcheck_path: String,
}

impl Config {
    /// Generate config from command line arguments and and environment variables.
    /// If config path is provided, read configuration from the file.
    pub fn initialize() -> Result<Self, BadConfigurationError> {
        let env_cli_config = EnvCLIConfig::from_args();
        let file_config: FileConfig =
            configuration::config::load_config_file(env_cli_config.config_path)
                .map_err(|e| BadConfigurationError { err: e.to_string() })?;

        let inspect_server_address: String = env_cli_config
            .inspect_server_address
            .or(file_config.inspect_server_address)
            .ok_or(BadConfigurationError {
                err: String::from("Must specify inspect server address"),
            })?;

        let server_manager_address: String = env_cli_config
            .server_manager_address
            .or(file_config.server_manager_address)
            .ok_or(BadConfigurationError {
                err: String::from("Must specify server manager address"),
            })?;

        let session_id: String = env_cli_config
            .session_id
            .or(file_config.session_id)
            .ok_or(BadConfigurationError {
                err: String::from("Must specify session id"),
            })?;

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
            return Err(BadConfigurationError {
                err: String::from(
                    "inspect path must be different from healthcheck path",
                ),
            });
        }

        Ok(Self {
            inspect_server_address,
            server_manager_address,
            session_id,
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

    /// Path to the config file
    #[structopt(long, env)]
    pub config_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct FileConfig {
    inspect_server_address: Option<String>,
    server_manager_address: Option<String>,
    session_id: Option<String>,
    inspect_path_prefix: Option<String>,
    healthcheck_path: Option<String>,
}

fn check_path_prefix(prefix: String) -> Result<String, BadConfigurationError> {
    let re = regex::Regex::new(r"^/[a-z]+$").unwrap();
    if re.is_match(&prefix) {
        Ok(prefix)
    } else {
        Err(BadConfigurationError {
            err: String::from(
                "invalid path prefix, it should be in the format `/[a-z]+`.",
            ),
        })
    }
}
