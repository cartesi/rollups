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

pub mod dapp_deployment {
    use serde::Deserialize;
    use state_fold_types::ethers::types::{Address, H256};

    #[derive(Clone, Debug, Deserialize)]
    pub struct DappDeployment {
        #[serde(rename = "address")]
        pub dapp_address: Address,

        #[serde(rename = "blockHash")]
        pub deploy_block_hash: H256,
    }
}

pub mod rollups_deployment {
    use serde::Deserialize;
    use state_fold_types::ethers::types::Address;

    #[derive(Clone, Debug, Deserialize)]
    struct DappDeployment {
        address: Address,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct RollupsDappsDeployment {
        #[serde(rename = "History")]
        history: DappDeployment,

        #[serde(rename = "Authority")]
        authority: DappDeployment,

        #[serde(rename = "InputBox")]
        input_box: DappDeployment,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct RollupsDeploymentJson {
        contracts: RollupsDappsDeployment,
    }

    #[derive(Clone, Debug)]
    pub struct RollupsDeployment {
        pub history_address: Address,
        pub authority_address: Address,
        pub input_box_address: Address,
    }

    impl From<RollupsDeploymentJson> for RollupsDeployment {
        fn from(r: RollupsDeploymentJson) -> Self {
            let contracts = r.contracts;
            Self {
                history_address: contracts.history.address,
                authority_address: contracts.authority.address,
                input_box_address: contracts.input_box.address,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::str::FromStr;

        use super::*;

        #[test]
        fn test_parse() {
            let history_address =
                Address::from_str("0xb6Eb78277C8a96Fb3f55BABef25eD0Bc5E5c95Fb")
                    .unwrap();
            let authority_address =
                Address::from_str("0xf3D8ce181a502B54512908a32780eaa9183Ef31a")
                    .unwrap();
            let input_box_address =
                Address::from_str("0x10dc33852b996A4C8A391d6Ed224FD89A3aD1ceE")
                    .unwrap();

            let data = r#"{
                "contracts": {
                    "History": {
                        "address": "0xb6Eb78277C8a96Fb3f55BABef25eD0Bc5E5c95Fb"
                    },

                    "Authority": {
                        "address": "0xf3D8ce181a502B54512908a32780eaa9183Ef31a"
                    },

                    "InputBox": {
                        "address": "0x10dc33852b996A4C8A391d6Ed224FD89A3aD1ceE"
                    }
                }
            }"#;

            let deployment: RollupsDeployment = {
                let deployment: RollupsDeploymentJson =
                    serde_json::from_str(data).unwrap();
                deployment.into()
            };

            assert_eq!(deployment.history_address, history_address);
            assert_eq!(deployment.authority_address, authority_address);
            assert_eq!(deployment.input_box_address, input_box_address);
        }
    }
}
