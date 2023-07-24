// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

//! Pristine merkle tree based on Cartesi machine-emulator implementation

use super::{
    get_concat_hash, Error, SizeOutOfRangeSnafu,
    WordSizeGreaterThanRootSizeSnafu,
};
use crate::hash::{Digest, Hash, Hasher};

/// Merkle tree where all leaves are zero
#[derive(Debug)]
pub struct Tree {
    log2_root_size: usize,
    log2_word_size: usize,
    hashes: Vec<Hash>,
}

impl Tree {
    /// Create a new pristine merkle tree
    ///
    /// - `log2_root_size`: Log2 of the size in bytes of the whole merkle tree.
    /// - `log2_word_size`: Log2 of the size in bytes of a single word.
    pub fn new(
        log2_root_size: usize,
        log2_word_size: usize,
    ) -> Result<Self, Error> {
        snafu::ensure!(
            log2_word_size <= log2_root_size,
            WordSizeGreaterThanRootSizeSnafu
        );
        let num_hashes = log2_root_size - log2_word_size + 1;
        let mut hashes = vec![];
        let mut hasher = Hasher::new();
        let word: Vec<u8> = vec![0; 1 << log2_word_size];
        hasher.update(&word);
        hashes.push(hasher.finalize_reset().into());
        for i in 1..num_hashes {
            hashes.push(get_concat_hash(
                &mut hasher,
                &hashes[i - 1],
                &hashes[i - 1],
            ));
        }
        Ok(Self {
            log2_root_size,
            log2_word_size,
            hashes,
        })
    }

    /// Get a hash for a sub-tree of the given size
    ///
    /// - `log2_size`: Log2 of the size in bytes of the subtree.
    pub fn get_hash(&self, log2_size: usize) -> Result<&Hash, Error> {
        snafu::ensure!(
            log2_size >= self.log2_word_size
                && log2_size <= self.log2_root_size,
            SizeOutOfRangeSnafu
        );
        Ok(&self.hashes[log2_size - self.log2_word_size])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_fails_to_create_a_tree_with_word_size_greater_than_root_size() {
        let err = Tree::new(2, 3).unwrap_err();
        assert_eq!(err, Error::WordSizeGreaterThanRootSize);
    }

    #[test]
    fn test_it_fails_to_get_hash_greater_than_root_size() {
        let tree = Tree::new(5, 3).unwrap();
        let err = tree.get_hash(6).unwrap_err();
        assert_eq!(err, Error::SizeOutOfRange);
    }

    #[test]
    fn test_it_fails_to_get_hash_smaller_than_word_size() {
        let tree = Tree::new(5, 3).unwrap();
        let err = tree.get_hash(2).unwrap_err();
        assert_eq!(err, Error::SizeOutOfRange);
    }

    #[test]
    fn test_it_creates_a_tree_with_root_size_equals_to_word_size() {
        let tree = Tree::new(5, 5).unwrap();
        assert_eq!(
            tree.get_hash(5).unwrap(),
            &Hash::decode("290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563")
        );
    }

    #[test]
    fn test_it_creates_a_tree_with_correct_hashes() {
        let tree = Tree::new(8, 3).unwrap();
        assert_eq!(
            tree.get_hash(3).unwrap(),
            &Hash::decode("011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce")
        );
        assert_eq!(
            tree.get_hash(4).unwrap(),
            &Hash::decode("4d9470a821fbe90117ec357e30bad9305732fb19ddf54a07dd3e29f440619254")
        );
        assert_eq!(
            tree.get_hash(5).unwrap(),
            &Hash::decode("ae39ce8537aca75e2eff3e38c98011dfe934e700a0967732fc07b430dd656a23")
        );
        assert_eq!(
            tree.get_hash(6).unwrap(),
            &Hash::decode("3fc9a15f5b4869c872f81087bb6104b7d63e6f9ab47f2c43f3535eae7172aa7f")
        );
        assert_eq!(
            tree.get_hash(7).unwrap(),
            &Hash::decode("17d2dd614cddaa4d879276b11e0672c9560033d3e8453a1d045339d34ba601b9")
        );
        assert_eq!(
            tree.get_hash(8).unwrap(),
            &Hash::decode("c37b8b13ca95166fb7af16988a70fcc90f38bf9126fd833da710a47fb37a55e6")
        );
    }
}
