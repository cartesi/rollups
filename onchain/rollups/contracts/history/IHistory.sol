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

/// @title History interface
interface IHistory {
    // Permissioned functions

    /// @notice Submit a claim.
    ///         The encoding of `_claimData` might vary
    ///         depending on the history implementation.
    /// @param _claimData Data for submitting a claim
    /// @dev The encoding of _claimData might vary depending on the history implementation
    /// @dev Should have access control.
    ///      Incorrect claim indices could raise two errors:
    ///      InvalidInputIndices if first index is posterior than last index or
    ///      UnclaimedInputs if first index is not the subsequent of previous claimed index.
    function submitClaim(bytes calldata _claimData) external;

    /// @notice Transfer ownership to another consensus.
    /// @param _consensus The new consensus
    /// @dev Should have access control.
    function migrateToConsensus(address _consensus) external;

    // Permissionless functions

    /// @notice Get a specific claim regarding a specific DApp.
    ///         The encoding of `_proofContext` might vary
    ///         depending on the history implementation.
    /// @param _dapp The DApp address
    /// @param _proofContext Data for retrieving the desired claim
    /// @return epochHash_ The claimed epoch hash
    /// @return firstInputIndex_ The index of the first input of the epoch in the input box
    /// @return lastInputIndex_ The index of the last input of the epoch in the input box
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
}
