// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IHistory} from "./IHistory.sol";

/// @title Simple History
///
/// @notice This contract stores claims for each DApp individually.
/// This means that, for each DApp, the contract stores an array of
/// `Claim` entries, where each `Claim` is composed of:
///
/// * An epoch hash (`bytes32`)
/// * A closed interval of input indices (`uint128`, `uint128`)
///
/// The contract guarantees that the first interval starts at index 0,
/// and that the following intervals don't have gaps or overlaps.
///
/// Furthermore, claims can only be submitted by the contract owner
/// through `submitClaim`, but can be retrieved by anyone with `getClaim`.
///
/// @dev This contract inherits OpenZeppelin's `Ownable` contract.
///      For more information on `Ownable`, please consult OpenZeppelin's official documentation.
contract History is IHistory, Ownable {
    struct Claim {
        bytes32 epochHash;
        uint128 firstIndex;
        uint128 lastIndex;
    }

    /// @notice Mapping from DApp address to number of claims.
    mapping(address => uint256) internal numClaims;

    /// @notice Mapping from DApp address and claim index to claim.
    /// @dev See the `getClaim` and `submitClaim` functions.
    mapping(address => mapping(uint256 => Claim)) internal claims;

    /// @notice A new claim regarding a specific DApp was submitted.
    /// @param dapp The address of the DApp
    /// @param claim The newly-submitted claim
    /// @dev MUST be triggered on a successful call to `submitClaim`.
    event NewClaimToHistory(address indexed dapp, Claim claim);

    /// @notice Raised when one tries to submit a claim whose first input index
    ///         is not less than or equal to its last input index.
    error InvalidInputIndices();

    /// @notice Raised when one tries to submit a claim that skips some input.
    ///         For example, when the 1st claim starts at index 5 (instead of 0)
    ///         or when the 1st claim ends at index 20 but the 2nd claim starts at
    ///         index 22 (instead of 21).
    error UnclaimedInputs();

    /// @notice Raised when one tries to retrieve a claim with an invalid index.
    error InvalidClaimIndex();

    /// @notice Creates a `History` contract.
    /// @param _owner The initial owner
    constructor(address _owner) {
        // constructor in Ownable already called `transferOwnership(msg.sender)`, so
        // we only need to call `transferOwnership(_owner)` if _owner != msg.sender
        if (_owner != msg.sender) {
            transferOwnership(_owner);
        }
    }

    /// @notice Submit a claim regarding a DApp.
    /// There are several requirements for this function to be called successfully.
    ///
    /// * `_claimData` MUST be well-encoded. In Solidity, it can be constructed
    ///   as `abi.encode(dapp, claim)`, where `dapp` is the DApp address (type `address`)
    ///   and `claim` is the claim structure (type `Claim`).
    ///
    /// * `firstIndex` MUST be less than or equal to `lastIndex`.
    ///   As a result, every claim MUST encompass AT LEAST one input.
    ///
    /// * If this is the DApp's first claim, then `firstIndex` MUST be `0`.
    ///   Otherwise, `firstIndex` MUST be the `lastClaim.lastIndex + 1`.
    ///   In other words, claims MUST NOT skip inputs.
    ///
    /// @inheritdoc IHistory
    /// @dev Emits a `NewClaimToHistory` event. Should have access control.
    ///      Incorrect claim input indices could raise two errors:
    ///      `InvalidInputIndices` if first index is posterior than last index or
    ///      `UnclaimedInputs` if first index is not the subsequent of previous claimed index or
    ///                        if the first index of the first claim is not zero.
    function submitClaim(
        bytes calldata _claimData
    ) external override onlyOwner {
        (address dapp, Claim memory claim) = abi.decode(
            _claimData,
            (address, Claim)
        );

        if (claim.firstIndex > claim.lastIndex) {
            revert InvalidInputIndices();
        }

        uint256 numDAppClaims = numClaims[dapp];

        if (
            claim.firstIndex !=
            (
                (numDAppClaims == 0)
                    ? 0
                    : (claims[dapp][numDAppClaims - 1].lastIndex + 1)
            )
        ) {
            revert UnclaimedInputs();
        }

        claims[dapp][numDAppClaims] = claim;
        numClaims[dapp] = numDAppClaims + 1;

        emit NewClaimToHistory(dapp, claim);
    }

    /// @notice Get a specific claim regarding a specific DApp.
    /// There are several requirements for this function to be called successfully.
    ///
    /// * `_proofContext` MUST be well-encoded. In Solidity, it can be constructed
    ///   as `abi.encode(claimIndex)`, where `claimIndex` is the claim index (type `uint256`).
    ///
    /// * `claimIndex` MUST be inside the interval `[0, n)` where `n` is the number of claims
    ///   that have been submitted to `_dapp` already.
    ///
    /// @inheritdoc IHistory
    /// @dev If `claimIndex` is not inside the interval `[0, n)`, then
    ///      an `InvalidClaimIndex` error is raised.
    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        uint256 claimIndex = abi.decode(_proofContext, (uint256));

        uint256 numDAppClaims = numClaims[_dapp];

        if (claimIndex >= numDAppClaims) {
            revert InvalidClaimIndex();
        }

        Claim memory claim = claims[_dapp][claimIndex];

        return (claim.epochHash, claim.firstIndex, claim.lastIndex);
    }

    /// @inheritdoc IHistory
    /// @dev Emits an `OwnershipTransferred` event. Should have access control.
    function migrateToConsensus(
        address _consensus
    ) external override onlyOwner {
        transferOwnership(_consensus);
    }
}
