// Copyright 2023 Cartesi Pte. Ltd.
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

use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use clap::Parser;
use std::time::Duration;

#[derive(Debug)]
pub struct RepositoryConfig {
    pub endpoint: String,
    pub connection_pool_size: u32,
    pub backoff: ExponentialBackoff,
}

#[derive(Debug, Parser)]
pub struct RepositoryCLIConfig {
    #[arg(long, env, default_value = "postgres")]
    postgres_user: String,

    #[arg(long, env)]
    postgres_password: Option<String>,

    #[arg(long, env)]
    postgres_password_file: Option<String>,

    #[arg(long, env, default_value = "127.0.0.1")]
    postgres_hostname: String,

    #[arg(long, env, default_value_t = 5432)]
    postgres_port: u16,

    #[arg(long, env, default_value = "postgres")]
    postgres_db: String,

    #[arg(long, env, default_value_t = 3)]
    postgres_connection_pool_size: u32,

    #[arg(long, env, default_value = "120000")]
    postgres_backoff_max_elapsed_duration: u64,
}

impl From<RepositoryCLIConfig> for RepositoryConfig {
    fn from(cli_config: RepositoryCLIConfig) -> RepositoryConfig {
        let password = if let Some(filename) = cli_config.postgres_password_file
        {
            if cli_config.postgres_password.is_some() {
                panic!("Both `postgres_password` and `postgres_password_file` arguments are set");
            }
            match std::fs::read_to_string(&filename) {
                Ok(password) => password,
                Err(e) => {
                    panic!("Failed to read password from file: {:?}", e);
                }
            }
        } else {
            cli_config
                .postgres_password
                .expect("Database Postgres password was not provided")
        };
        let endpoint = format!(
            "postgres://{}:{}@{}:{}/{}",
            urlencoding::encode(&cli_config.postgres_user),
            urlencoding::encode(&password),
            urlencoding::encode(&cli_config.postgres_hostname),
            cli_config.postgres_port,
            urlencoding::encode(&cli_config.postgres_db)
        );
        let connection_pool_size = cli_config.postgres_connection_pool_size;
        let backoff_max_elapsed_duration = Duration::from_millis(
            cli_config.postgres_backoff_max_elapsed_duration,
        );
        let backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(backoff_max_elapsed_duration))
            .build();
        RepositoryConfig {
            endpoint,
            connection_pool_size,
            backoff,
        }
    }
}
