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

pub use sha3::Digest;
use sha3::{digest::Output, Keccak256};

pub const HASH_SIZE: usize = 32;

pub type Hasher = Keccak256;

#[derive(Clone, PartialEq, Eq)]
pub struct Hash {
    data: [u8; HASH_SIZE],
}

impl Hash {
    pub fn data(&self) -> &[u8; HASH_SIZE] {
        &self.data
    }

    #[cfg(test)]
    pub fn decode(s: &str) -> Hash {
        Hash {
            data: hex::decode(&s)
                .expect("invalid hex string")
                .try_into()
                .expect("cannot fail"),
        }
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self {
            data: [0; HASH_SIZE],
        }
    }
}

impl From<Output<Keccak256>> for Hash {
    fn from(arr: Output<Keccak256>) -> Hash {
        Hash { data: arr.into() }
    }
}

impl From<[u8; HASH_SIZE]> for Hash {
    fn from(data: [u8; HASH_SIZE]) -> Hash {
        Hash { data }
    }
}

impl TryFrom<Vec<u8>> for Hash {
    type Error = Vec<u8>;

    fn try_from(v: Vec<u8>) -> Result<Hash, Vec<u8>> {
        Ok(Hash {
            data: v.try_into()?,
        })
    }
}

impl From<Hash> for Vec<u8> {
    fn from(hash: Hash) -> Vec<u8> {
        Vec::from(hash.data)
    }
}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.data))
    }
}
