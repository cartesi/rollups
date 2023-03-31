// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Canonical Machine Constants
pragma solidity ^0.8.8;

library CanonicalMachine {
    // Log base 2 of size in bytes
    type Log2Size is uint64;

    // Machine word size (8 bytes)
    Log2Size constant WORD_LOG2_SIZE = Log2Size.wrap(3);

    // Machine address space size (2^64 bytes)
    Log2Size constant MACHINE_LOG2_SIZE = Log2Size.wrap(64);

    // Keccak-256 output size (32 bytes)
    Log2Size constant KECCAK_LOG2_SIZE = Log2Size.wrap(5);

    // Maximum input size (32 megabytes)
    Log2Size constant INPUT_MAX_LOG2_SIZE = Log2Size.wrap(25);

    // Maximum voucher metadata memory range (2 megabytes)
    Log2Size constant VOUCHER_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    // Maximum notice metadata memory range (2 megabytes)
    Log2Size constant NOTICE_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    // Maximum epoch voucher memory range (128 megabytes)
    Log2Size constant EPOCH_VOUCHER_LOG2_SIZE = Log2Size.wrap(37);

    // Maximum epoch notice memory range (128 megabytes)
    Log2Size constant EPOCH_NOTICE_LOG2_SIZE = Log2Size.wrap(37);

    /// @notice Convert a Log2Size value into its underlying uint64 value
    /// @param s the Log2Size value
    function uint64OfSize(Log2Size s) internal pure returns (uint64) {
        return Log2Size.unwrap(s);
    }

    /// @notice Return the position of an intra memory range on a memory range
    //          with contents with the same size
    /// @param _index index of intra memory range
    /// @param _log2Size size of intra memory range
    function getIntraMemoryRangePosition(
        uint64 _index,
        Log2Size _log2Size
    ) internal pure returns (uint64) {
        return _index << Log2Size.unwrap(_log2Size);
    }
}
