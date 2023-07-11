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

use state_fold_types::ethers::types::Address;

use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Default)]
pub struct UserData {
    addresses: HashSet<Arc<Address>>,
}

impl UserData {
    pub fn get(&mut self, address: Address) -> Arc<Address> {
        // Method `get_or_insert` of HashSet is still unstable
        match self.addresses.get(&address) {
            Some(s) => Arc::clone(s),
            None => {
                let s = Arc::new(address);
                assert!(self.addresses.insert(s.clone()));
                s
            }
        }
    }
}
