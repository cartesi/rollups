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
