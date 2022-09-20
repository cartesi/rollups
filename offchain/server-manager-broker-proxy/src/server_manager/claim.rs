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

use sha3::{Digest, Keccak256};

pub fn compute_claim_hash(
    machine_state_hash: &[u8],
    vouchers_metadata_hash: &[u8],
    notices_metadata_hash: &[u8],
) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(machine_state_hash);
    hasher.update(vouchers_metadata_hash);
    hasher.update(notices_metadata_hash);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::compute_claim_hash;

    #[test_log::test]
    fn test_claim_hash() {
        let hash = hex::decode(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap();
        let claim = compute_claim_hash(&hash, &hash, &hash);
        let expected = hex::decode(
            "8590bbc3ea43e28e8624fb1a2d59aaca701a5517e08511c4a14d9037de6f6086",
        )
        .unwrap();
        assert_eq!(&expected, &claim);
    }
}
