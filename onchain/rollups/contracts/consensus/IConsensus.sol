// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Consensus interface
pragma solidity ^0.8.13;

interface IConsensus {
    /// @notice An application has joined the consensus' validation set
    /// @param application The application
    /// @dev MUST be triggered on a successful call to `join()`
    event ApplicationJoined(address application);

    /// @notice Get a claim
    /// @param _dapp The DApp
    /// @param _proofContext Data for retrieving the desired claim
    /// @return epochHash_ The epoch hash contained in the claim
    /// @return firstInputIndex_ The index of the first input in the input box for which the epoch hash is valid
    /// @return lastInputIndex_ The index of the last input in the input box for which the epoch hash is valid
    /// @dev The encoding of _proofContext might vary depending on the implementation
    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    )
        external
        view
        returns (
            bytes32 epochHash_,
            uint256 firstInputIndex_,
            uint256 lastInputIndex_
        );

    /// @notice Join the consensus' validation set
    /// @dev This function should be called by a DApp when it migrates to this consensus
    /// @dev MUST fire the `ApplicationJoined` event with the message sender as argument
    function join() external;
}
