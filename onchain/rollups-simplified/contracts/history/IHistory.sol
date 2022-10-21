// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title History interface
pragma solidity ^0.8.13;

interface IHistory {
    // Events

    /// @notice A new claim was submitted
    /// @param claimData The claim data that was submitted (must match the _claimData
    ///                  parameter to the submitClaim function).
    /// @dev The encoding of claim might vary depending on the history implementation
    event NewClaim(bytes claimData);

    // Permissioned functions

    /// @notice Submit a claim
    /// @param _claimData Data for submitting a claim
    /// @dev The encoding of _claimData might vary depending on the history implementation
    /// @dev Should have access control
    function submitClaim(bytes calldata _claimData) external;

    /// @notice Migrate to a new consensus
    /// @param _consensus The new consensus
    /// @dev Should have access control
    function migrateToConsensus(address _consensus) external;

    // Permissionless functions

    /// @notice Get the epoch hash for a given DApp from a claim
    /// @param _dapp The DApp
    /// @param _claimQuery Data for querying the desired claim
    /// @return epochHash_ The epoch hash contained in the claim
    /// @return inputIndex_ The index of the input in the input box
    /// @return epochInputIndex_ The offset between the input in the input box
    //                           and the first input of the epoch in the input box
    /// @dev The encoding of _claimQuery might vary depending on the history implementation
    function getEpochHash(address _dapp, bytes calldata _claimQuery)
        external
        view
        returns (
            bytes32 epochHash_,
            uint256 inputIndex_,
            uint256 epochInputIndex_
        );
}
