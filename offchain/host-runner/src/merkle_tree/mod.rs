// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pub mod complete;
pub mod pristine;
pub mod proof;

use snafu::Snafu;

use crate::hash::{Digest, Hash, Hasher};

#[derive(Debug, Snafu, PartialEq)]
pub enum Error {
    #[snafu(display("log2_target_size is greater than log2_root_size"))]
    TargetSizeGreaterThanRootSize,
    #[snafu(display("log2_leaf_size is greater than log2_root_size"))]
    LeafSizeGreaterThanRootSize,
    #[snafu(display("log2_word_size is greater than log2_leaf_size"))]
    WordSizeGreaterThanLeafSize,
    #[snafu(display("log2_word_size is greater than log2_root_size"))]
    WordSizeGreaterThanRootSize,
    #[snafu(display("tree is too large for address type"))]
    TreeTooLarge,
    #[snafu(display("tree is full"))]
    TreeIsFull,
    #[snafu(display("too many leaves"))]
    TooManyLeaves,
    #[snafu(display("log2_size is out of range"))]
    SizeOutOfRange,
    #[snafu(display("address is misaligned"))]
    MisalignedAddress,
}

fn get_concat_hash(hasher: &mut Hasher, left: &Hash, right: &Hash) -> Hash {
    hasher.reset();
    hasher.update(left.data());
    hasher.update(right.data());
    hasher.finalize_reset().into()
}
