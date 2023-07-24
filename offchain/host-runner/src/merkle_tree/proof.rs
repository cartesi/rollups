// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

//! Merkle tree proof based on Cartesi machine-emulator implementation

use super::{Error, SizeOutOfRangeSnafu, TargetSizeGreaterThanRootSizeSnafu};

use crate::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub target_address: usize,
    pub log2_target_size: usize,
    pub target_hash: Hash,
    pub log2_root_size: usize,
    pub root_hash: Hash,
    pub sibling_hashes: Vec<Hash>,
}

/// Merkle tree proof structure
///
/// This structure holds a proof that the node spanning a log2_target_size at a given address in
/// the tree has a certain hash.
impl Proof {
    /// Constructs a merkle_tree_proof object and allocates room for the sibling hashes
    pub fn new(
        target_address: usize,
        log2_target_size: usize,
        target_hash: Hash,
        log2_root_size: usize,
        root_hash: Hash,
    ) -> Result<Self, Error> {
        snafu::ensure!(
            log2_target_size <= log2_root_size,
            TargetSizeGreaterThanRootSizeSnafu
        );
        Ok(Self {
            target_address,
            log2_target_size,
            target_hash,
            log2_root_size,
            root_hash,
            sibling_hashes: vec![
                Hash::default();
                log2_root_size - log2_target_size
            ],
        })
    }

    /// Modify hash corresponding to log2_size in the list of siblings.
    pub fn set_sibling_hash(
        &mut self,
        hash: Hash,
        log2_size: usize,
    ) -> Result<(), Error> {
        let index = self.log2_size_to_index(log2_size)?;
        self.sibling_hashes[index] = hash;
        Ok(())
    }

    /// Converts log2_size to index into siblings array
    fn log2_size_to_index(&self, log2_size: usize) -> Result<usize, Error> {
        snafu::ensure!(log2_size < self.log2_root_size, SizeOutOfRangeSnafu);
        snafu::ensure!(log2_size >= self.log2_target_size, SizeOutOfRangeSnafu);
        let index = log2_size - self.log2_target_size;
        snafu::ensure!(index < self.sibling_hashes.len(), SizeOutOfRangeSnafu);
        Ok(index)
    }
}
