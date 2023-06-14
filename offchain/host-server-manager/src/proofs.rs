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

use crate::hash::{Digest, Hash, Hasher};
use crate::merkle_tree::{self, complete::Tree, proof::Proof};

const LOG2_ROOT_SIZE: usize = 16 + LOG2_HASH_SIZE;
const LOG2_WORD_SIZE: usize = 3;
const LOG2_HASH_SIZE: usize = 5;
const WORD_SIZE: usize = 1 << LOG2_WORD_SIZE;
const WORDS_PER_HASH: usize = 1 << (LOG2_HASH_SIZE - LOG2_WORD_SIZE);

/// Trait to be implemented by vouchers and notices
pub trait Proofable {
    fn get_hash(&self) -> &Hash;
    fn set_proof(&mut self, proof: Proof);
}

/// Update the merkle proofs of every proofable in the array and return the merkle-tree's root hash
pub fn compute_proofs(proofables: &mut [impl Proofable]) -> Result<Hash, merkle_tree::Error> {
    let mut hasher = Hasher::new();
    let mut leaves: Vec<Hash> = vec![];
    for proofable in proofables.iter() {
        let hash = proofable.get_hash();
        for word in 0..WORDS_PER_HASH {
            let start = word * WORD_SIZE;
            let end = start + WORD_SIZE;
            hasher.update(&hash.data()[start..end]);
            let word_hash = hasher.finalize_reset().into();
            leaves.push(word_hash);
        }
    }
    let tree = Tree::new_from_leaves(LOG2_ROOT_SIZE, LOG2_WORD_SIZE, LOG2_WORD_SIZE, leaves)?;
    for (i, proofable) in proofables.iter_mut().enumerate() {
        let proof = tree.get_proof(i * (1 << LOG2_HASH_SIZE), LOG2_HASH_SIZE)?;
        proofable.set_proof(proof);
    }
    Ok(tree.get_root_hash().clone())
}
