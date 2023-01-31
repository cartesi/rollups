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

use crate::Address;
use clap::Parser;
use serde_json::Value;
use std::{fs::File, io::BufReader};

/// DApp metadata used to define the stream keys
#[derive(Debug, Clone)]
pub struct DAppMetadata {
    pub chain_id: u64,
    pub dapp_id: Address,
}

/// CLI configuration used to generate the DApp metadata
#[derive(Debug, Parser)]
pub struct DAppMetadataCLIConfig {
    /// Chain identifier
    #[arg(long, env, default_value = "0")]
    chain_id: u64,

    /// Address of rollups dapp
    #[arg(long, env)]
    dapp_contract_address: Option<String>,

    /// Path to file with address of rollups dapp
    #[arg(long, env)]
    dapp_contract_address_file: Option<String>,
}

impl From<DAppMetadataCLIConfig> for DAppMetadata {
    fn from(cli_config: DAppMetadataCLIConfig) -> DAppMetadata {
        let dapp_contract_address_raw = match cli_config.dapp_contract_address {
            Some(address) => address,
            None => {
                let path = cli_config
                    .dapp_contract_address_file
                    .expect("Configuration missing dapp address");
                let file = File::open(path).expect("Dapp json read file error");
                let reader = BufReader::new(file);
                let mut json: Value = serde_json::from_reader(reader)
                    .expect("Dapp json parse error");
                match json["address"].take() {
                    Value::String(s) => s,
                    Value::Null => panic!("Configuration missing dapp address"),
                    _ => panic!("Dapp json wrong type error"),
                }
            }
        };

        let dapp_contract_address: [u8; 20] =
            hex::decode(&dapp_contract_address_raw[2..])
                .expect("Dapp json parse error")
                .try_into()
                .expect("Dapp address with wrong size");

        DAppMetadata {
            chain_id: cli_config.chain_id,
            dapp_id: dapp_contract_address.into(),
        }
    }
}

/// Declares a struct that implements the BrokerStream interface
/// The generated key has the format `chain-<chain_id>:dapp-<dapp_id>:<key>`
macro_rules! decl_broker_stream {
    ($stream: ident, $payload: ty, $key: literal) => {
        #[derive(Debug)]
        pub struct $stream {
            key: String,
        }

        impl crate::broker::BrokerStream for $stream {
            type Payload = $payload;

            fn key(&self) -> &str {
                &self.key
            }
        }

        impl $stream {
            pub fn new(metadata: &crate::rollups_stream::DAppMetadata) -> Self {
                Self {
                    key: format!(
                        "chain-{}:dapp-{}:{}",
                        metadata.chain_id,
                        hex::encode(metadata.dapp_id.inner()),
                        $key
                    ),
                }
            }
        }
    };
}

pub(crate) use decl_broker_stream;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ADDRESS_SIZE;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct MockPayload;

    decl_broker_stream!(MockStream, MockPayload, "rollups-mock");

    #[test]
    fn it_generates_the_key() {
        let metadata = DAppMetadata {
            chain_id: 123,
            dapp_id: Address::new([0xfa; ADDRESS_SIZE]),
        };
        let stream = MockStream::new(&metadata);
        assert_eq!(stream.key, "chain-123:dapp-fafafafafafafafafafafafafafafafafafafafa:rollups-mock");
    }
}
