// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Canonical Machine Constants
pragma solidity ^0.8.13;

library CanonicalMachine {
    type Log2Size is uint64;

    Log2Size constant INPUT_MAX_LOG2_SIZE = Log2Size.wrap(25);

    // cartesi machine word log2 size
    Log2Size constant WORD_LOG2_SIZE = Log2Size.wrap(3);

    // keccak log2 size
    Log2Size constant KECCAK_LOG2_SIZE = Log2Size.wrap(5);

    // max size of voucher metadata memory range 32 * (2^16) bytes
    Log2Size constant VOUCHER_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    // max size of notice metadata memory range 32 * (2^16) bytes
    Log2Size constant NOTICE_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    // max size of epoch voucher memory range 32 * (2^32) bytes
    Log2Size constant EPOCH_VOUCHER_LOG2_SIZE = Log2Size.wrap(37);

    // max size of epoch notice memory range 32 * (2^32) bytes
    Log2Size constant EPOCH_NOTICE_LOG2_SIZE = Log2Size.wrap(37);

    // cartesi machine address space log2 size
    Log2Size constant MACHINE_LOG2_SIZE = Log2Size.wrap(64);

    function uint64OfSize(Log2Size s) internal pure returns (uint64) {
        return Log2Size.unwrap(s);
    }

    /// @notice returns the position of a intra memory range on a memory range
    //          with  contents with the same size
    /// @param _index index of intra memory range
    /// @param _log2Size of intra memory range
    function getIntraMemoryRangePosition(uint64 _index, Log2Size _log2Size)
        public
        pure
        returns (uint64)
    {
        return _index << Log2Size.unwrap(_log2Size);
    }
}
