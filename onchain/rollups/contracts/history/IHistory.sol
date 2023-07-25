// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

/// @title History interface
interface IHistory {
    // Permissioned functions

    /// @notice Submit a claim.
    ///         The encoding of `_claimData` might vary
    ///         depending on the history implementation.
    /// @param _claimData Data for submitting a claim
    /// @dev Should have access control.
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
