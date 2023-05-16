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

use rollups_events::{Hash, HASH_SIZE};
use sha3::{Digest, Keccak256};

pub fn compute_epoch_hash(
    machine_state_hash: &Hash,
    vouchers_metadata_hash: &Hash,
    notices_metadata_hash: &Hash,
) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(machine_state_hash.inner());
    hasher.update(vouchers_metadata_hash.inner());
    hasher.update(notices_metadata_hash.inner());
    let data: [u8; HASH_SIZE] = hasher.finalize().into();
    Hash::new(data)
}

#[cfg(test)]
mod tests {
    use super::{compute_epoch_hash, Hash};

    #[test_log::test]
    fn test_claim_hash() {
        let hash = Hash::new(
            hex::decode(
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )
            .unwrap()
            .try_into()
            .unwrap()
        );
        let claim = compute_epoch_hash(&hash, &hash, &hash);
        let expected = Hash::new(
            hex::decode(
                "8590bbc3ea43e28e8624fb1a2d59aaca701a5517e08511c4a14d9037de6f6086",
            )
            .unwrap()
            .try_into()
            .unwrap()
        );
        assert_eq!(expected, claim);
    }
}
