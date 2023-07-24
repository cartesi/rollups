// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use crate::Address;
use clap::Parser;
use prometheus_client::encoding::EncodeLabelSet;
use serde_json::Value;
use std::{fs::File, io::BufReader};

/// DApp metadata used to define the stream keys
#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, EncodeLabelSet)]
pub struct DAppMetadata {
    pub chain_id: u64,
    pub dapp_address: Address,
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
            dapp_address: dapp_contract_address.into(),
        }
    }
}

/// Declares a struct that implements the BrokerStream interface
/// The generated key has the format `{chain-<chain_id>:dapp-<dapp_address>}:<key>`.
/// The curly braces define a hash tag to ensure that all of a dapp's streams
/// are located in the same node when connected to a Redis cluster.
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
                        "{{chain-{}:dapp-{}}}:{}",
                        metadata.chain_id,
                        hex::encode(metadata.dapp_address.inner()),
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
            dapp_address: Address::new([0xfa; ADDRESS_SIZE]),
        };
        let stream = MockStream::new(&metadata);
        assert_eq!(stream.key, "{chain-123:dapp-fafafafafafafafafafafafafafafafafafafafa}:rollups-mock");
    }
}
