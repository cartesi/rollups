// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

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
    /// @param inboxInputIndex The index of the input in the input box
    /// @return The input hash
    function computeInputHash(
        address sender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        bytes calldata input,
        uint256 inboxInputIndex
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
                inboxInputIndex // input index in the input box
            )
        );

        bytes32 keccakInput = keccak256(input);

        return keccak256(abi.encode(keccakMetadata, keccakInput));
    }
}
