// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

/// @title Canonical Machine Constants Library
///
/// @notice Defines several constants related to the reference implementation
/// of the RISC-V machine that runs Linux, also known as the "Cartesi Machine".
library CanonicalMachine {
    /// @notice Base-2 logarithm of number of bytes.
    type Log2Size is uint64;

    /// @notice Machine word size (8 bytes).
    Log2Size constant WORD_LOG2_SIZE = Log2Size.wrap(3);

    /// @notice Machine address space size (2^64 bytes).
    Log2Size constant MACHINE_LOG2_SIZE = Log2Size.wrap(64);

    /// @notice Keccak-256 output size (32 bytes).
    Log2Size constant KECCAK_LOG2_SIZE = Log2Size.wrap(5);

    /// @notice Maximum input size (~2 megabytes).
    /// @dev The offset and size fields use up the extra 64 bytes.
    uint256 constant INPUT_MAX_SIZE = (1 << 21) - 64;

    /// @notice Maximum voucher metadata memory range (2 megabytes).
    Log2Size constant VOUCHER_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    /// @notice Maximum notice metadata memory range (2 megabytes).
    Log2Size constant NOTICE_METADATA_LOG2_SIZE = Log2Size.wrap(21);

    /// @notice Maximum epoch voucher memory range (128 megabytes).
    Log2Size constant EPOCH_VOUCHER_LOG2_SIZE = Log2Size.wrap(37);

    /// @notice Maximum epoch notice memory range (128 megabytes).
    Log2Size constant EPOCH_NOTICE_LOG2_SIZE = Log2Size.wrap(37);

    /// @notice Unwrap `s` into its underlying uint64 value.
    /// @param s Base-2 logarithm of some number of bytes
    function uint64OfSize(Log2Size s) internal pure returns (uint64) {
        return Log2Size.unwrap(s);
    }

    /// @notice Return the position of an intra memory range on a memory range
    ///         with contents with the same size.
    /// @param index Index of intra memory range
    /// @param log2Size Base-2 logarithm of intra memory range size
    function getIntraMemoryRangePosition(
        uint64 index,
        Log2Size log2Size
    ) internal pure returns (uint64) {
        return index << Log2Size.unwrap(log2Size);
    }
}
