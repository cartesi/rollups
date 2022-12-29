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

use clap::Parser;
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub redis_endpoint: String,
    pub chain_id: u64,
    pub dapp_contract_address: [u8; 20],
    pub consume_timeout: usize,
}

impl BrokerConfig {
    pub fn parse_from_cli(
        cli_config: BrokerCLIConfig,
    ) -> Result<Self, BrokerConfigError> {
        let dapp_contract_address_raw = match cli_config.dapp_contract_address {
            Some(address) => address,
            None => {
                let path = cli_config
                    .dapp_contract_address_file
                    .ok_or(snafu::NoneError)
                    .context(MissingDappAddressSnafu)?;
                let file = File::open(path).context(DappJsonReadFileSnafu)?;
                let reader = BufReader::new(file);
                let mut json: Value = serde_json::from_reader(reader)
                    .context(DappJsonParseSnafu)?;
                match json["address"].take() {
                    Value::String(s) => s,
                    Value::Null => return MissingDappAddressSnafu.fail(),
                    _ => return DappJsonWrongTypeSnafu.fail(),
                }
            }
        };

        let dapp_contract_address =
            hex::decode(&dapp_contract_address_raw[2..])
                .context(DappAddressParseSnafu)?
                .try_into()
                .map_err(|_| BrokerConfigError::DappAddressSizeError {})?;

        Ok(Self {
            redis_endpoint: cli_config.redis_endpoint,
            chain_id: cli_config.chain_id,
            dapp_contract_address,
            consume_timeout: cli_config.consume_timeout,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum BrokerConfigError {
    #[snafu(display("Configuration missing dapp address"))]
    MissingDappAddress {},

    #[snafu(display("Dapp json read file error"))]
    DappJsonReadFileError { source: std::io::Error },

    #[snafu(display("Dapp json parse error"))]
    DappJsonParseError { source: serde_json::Error },

    #[snafu(display("Dapp json wrong type error"))]
    DappJsonWrongTypeError {},

    #[snafu(display("Dapp address string parse error"))]
    DappAddressParseError { source: hex::FromHexError },

    #[snafu(display("Dapp address with wrong size"))]
    DappAddressSizeError {},
}

#[derive(Parser, Debug)]
#[command(name = "broker")]
pub struct BrokerCLIConfig {
    /// Redis address
    #[arg(long, env, default_value = "redis://127.0.0.1:6379")]
    redis_endpoint: String,

    /// Chain identifier
    #[arg(long, env, default_value = "0")]
    chain_id: u64,

    /// Address of rollups dapp
    #[arg(long, env)]
    dapp_contract_address: Option<String>,

    /// Path to file with address of rollups dapp
    #[arg(long, env)]
    dapp_contract_address_file: Option<String>,

    /// Timeout when consuming input events (in millis)
    #[arg(long, env, default_value = "5000")]
    consume_timeout: usize,
}
