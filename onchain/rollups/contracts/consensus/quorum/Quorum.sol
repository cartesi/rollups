// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {AccessControlEnumerable} from "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {PaymentSplitter} from "@openzeppelin/contracts/finance/PaymentSplitter.sol";

import {AbstractConsensus} from "../AbstractConsensus.sol";
import {IHistory} from "../../history/IHistory.sol";

/// @title Quorum consensus
/// @notice A consensus model controlled by a small set of addresses, the validators.
/// @dev This contract uses several OpenZeppelin contracts:
/// `AccessControlEnumerable`, `EnumerableSet`, and `PaymentSplitter`.
/// For more information on those, please consult OpenZeppelin's official documentation.
contract Quorum is AbstractConsensus, AccessControlEnumerable, PaymentSplitter {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @notice The validator role.
    /// @dev Only validators can submit claims.
    bytes32 public constant VALIDATOR_ROLE = keccak256("VALIDATOR_ROLE");

    /// @notice The history contract.
    /// @dev See the `getHistory` function.
    IHistory internal immutable history;

    /// @notice For each claim, the set of validators that agree
    /// that it should be submitted to the history contract.
    mapping(bytes => EnumerableSet.AddressSet) internal yeas;

    /// @notice Construct a Quorum consensus
    /// @param _validators the list of validators
    /// @param _shares the list of shares
    /// @param _history the history contract
    constructor(
        address[] memory _validators,
        uint256[] memory _shares,
        IHistory _history
    ) PaymentSplitter(_validators, _shares) {
        // Iterate through the array of validators,
        // and grant to each the validator role.
        for (uint256 i; i < _validators.length; ++i) {
            grantRole(VALIDATOR_ROLE, _validators[i]);
        }

        // Set history.
        history = _history;
    }

    /// @notice Submits a claim for voting.
    ///         If this is the claim that reaches the majority, then
    ///         the claim is submitted to the history contract.
    ///         The encoding of `_claimData` might vary depending on the
    ///         implementation of the current history contract.
    /// @param _claimData Data for submitting a claim
    /// @dev Can only be called by a validator,
    ///      and the `Quorum` contract must have ownership over
    ///      its current history contract.
    function submitClaim(
        bytes calldata _claimData
    ) external onlyRole(VALIDATOR_ROLE) {
        // Get the set of validators in favour of the claim
        EnumerableSet.AddressSet storage claimYeas = yeas[_claimData];

        // Add the message sender to such set.
        // If the `add` function returns `true`,
        // then the message sender was not in the set.
        if (claimYeas.add(msg.sender)) {
            // Get the number of validators in favour of the claim,
            // taking into account the message sender as well.
            uint256 numOfVotesInFavour = claimYeas.length();

            // Get the number of validators in the quorum.
            uint256 quorumSize = getRoleMemberCount(VALIDATOR_ROLE);

            // If this claim has now just over half of the quorum's approval,
            // then we can submit it to the history contract.
            if (numOfVotesInFavour == 1 + quorumSize / 2) {
                history.submitClaim(_claimData);
            }
        }
    }

    /// @notice Get the history contract.
    /// @return The history contract
    function getHistory() external view returns (IHistory) {
        return history;
    }

    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        return history.getClaim(_dapp, _proofContext);
    }
}
