// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {CanonicalMachine} from "../common/CanonicalMachine.sol";

/// @title Input Library
library LibInput {
    using CanonicalMachine for CanonicalMachine.Log2Size;

    /// @notice Raised when input is larger than the machine limit.
    error InputSizeExceedsLimit();

    /// @notice Summarize input data in a single hash.
    /// @param sender `msg.sender`
    /// @param blockNumber `block.number`
    /// @param blockTimestamp `block.timestamp`
    /// @param input The input blob
    /// @param inputIndex The index of the input in the input box
    /// @return The input hash
    function computeInputHash(
        address sender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        bytes calldata input,
        uint256 inputIndex
    ) internal pure returns (bytes32) {
        if (input.length > CanonicalMachine.INPUT_MAX_SIZE) {
            revert InputSizeExceedsLimit();
        }

        bytes32 keccakMetadata = keccak256(
            abi.encode(
                sender,
                blockNumber,
                blockTimestamp,
                0, //TODO decide how to deal with epoch index
                inputIndex // input index in the input box
            )
        );

        bytes32 keccakInput = keccak256(input);

        return keccak256(abi.encode(keccakMetadata, keccakInput));
    }
}
