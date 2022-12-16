// Copyright (C) 2022 Cartesi Pte. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use state_fold_types::{
    ethabi::ethereum_types::{Address, Bloom, H256, U256, U64},
    Block,
};
use std::{str::FromStr, sync::Arc};
use types::foldables::{
    authority::RollupsState,
    claims::{History, HistoryInitialState},
    input_box::{DAppInputBox, Input, InputBox, InputBoxInitialState},
};

#[allow(dead_code)]
pub async fn get_test_block_state_01() -> Arc<RollupsState> {
    let input_box = {
        let input_0 = {
            let block = Arc::new(Block {
                hash: H256::zero(),
                number: U64::from(34),
                parent_hash: H256::zero(),
                timestamp: U256::zero(),
                logs_bloom: Bloom::zero(),
            });

            Input {
                sender: Arc::new(
                    Address::from_str(
                        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                    )
                    .expect("valid address string"),
                ),
                payload: vec![
                    67, 82, 69, 65, 84, 69, 32, 84, 65, 66, 76, 69, 32, 80,
                    101, 114, 115, 111, 110, 115, 32, 40, 110, 97, 109, 101,
                    32, 116, 101, 120, 116, 44, 32, 97, 103, 101, 32, 105, 110,
                    116, 41,
                ],
                block_added: block,
                dapp: Arc::new(Address::default()),
            }
        };

        let input_1 = {
            let block = Arc::new(Block {
                hash: H256::zero(),
                number: U64::from(36),
                parent_hash: H256::zero(),
                timestamp: U256::zero(),
                logs_bloom: Bloom::zero(),
            });

            Input {
                sender: Arc::new(
                    Address::from_str(
                        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                    )
                    .expect("valid address string"),
                ),
                payload: vec![
                    83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79, 77, 32, 80,
                    101, 114, 115, 111, 110, 115,
                ],
                block_added: block,
                dapp: Arc::new(Address::default()),
            }
        };

        let dapp_input_box = DAppInputBox {
            inputs: im::vector![
                Arc::new(input_0.clone()),
                Arc::new(input_0.clone()),
                Arc::new(input_0.clone()),
                Arc::new(input_1.clone()),
                Arc::new(input_1.clone()),
            ],
        };

        InputBox {
            input_box_address: Arc::new(Address::default()),
            dapp_input_boxes: Arc::new(im::HashMap::unit(
                Arc::new(Address::default()),
                Arc::new(dapp_input_box),
            )),
        }
    };

    let history = History {
        history_address: Arc::new(Address::default()),
        dapp_claims: Arc::new(im::HashMap::new()),
    };

    Arc::new(RollupsState {
        input_box_initial_state: Arc::new(InputBoxInitialState {
            input_box_address: Arc::new(Address::default()),
        }),
        input_box: Arc::new(input_box),

        history_initial_state: Arc::new(HistoryInitialState {
            history_address: Arc::new(Address::default()),
        }),
        history: Arc::new(history),
    })
}
