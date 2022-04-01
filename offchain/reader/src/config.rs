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

use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "reader_config", about = "Configuration for reader")]
pub struct ReaderCliConfig {
    /// Postgres database url
    #[structopt(long = "--username", env = "DB_USER")]
    pub db_username: String,
    #[structopt(long = "--password", env = "DB_PASSWORD")]
    pub db_password: String,
    #[structopt(
        long = "--db-host",
        env = "DB_HOST",
        default_value = "127.0.0.1"
    )]
    pub db_host: String,
    #[structopt(long = "--db-port", env = "DB_PORT", default_value = "5432")]
    pub db_port: String,
    #[structopt(
        long = "--db-name",
        env = "DB_NAME",
        default_value = "postgres"
    )]
    pub db_name: String,
    #[structopt(
        long = "--db-testname",
        env = "DB_TEST_NAME",
        default_value = "postgres"
    )]
    pub db_test_name: String,
    #[structopt(
        long = "--host",
        env = "GRAPHQL_HOST",
        default_value = "127.0.0.1"
    )]
    pub host: String,
    #[structopt(long = "--port", env = "GRAPHQL_PORT", default_value = "4000")]
    pub port: String,
}

pub struct ReaderConfig {
    pub db_host: String,
    pub db_port: u16,
    pub db_username: String,
    pub db_password: String,
    pub db_name: String,
    pub db_test_name: String,
    pub graphql_host: String,
    pub graphql_port: u16,
}

impl ReaderConfig {
    pub fn initialize() -> crate::error::Result<Self> {
        let reader_cli_config = ReaderCliConfig::from_args();

        println!("CONFIG {:?}", &reader_cli_config);

        Ok(ReaderConfig {
            db_host: reader_cli_config.db_host.clone(),
            db_port: u16::from_str_radix(&reader_cli_config.db_port, 10)
                .expect("valid database port"),
            db_username: reader_cli_config.db_username.to_string(),
            db_password: reader_cli_config.db_password.to_string(),
            db_name: reader_cli_config.db_name.clone(),
            db_test_name: reader_cli_config.db_test_name.clone(),
            graphql_host: reader_cli_config.host.clone(),
            graphql_port: u16::from_str_radix(&reader_cli_config.port, 10)
                .expect("valid port"),
        })
    }
}
