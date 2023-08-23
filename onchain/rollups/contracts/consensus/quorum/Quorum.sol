// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {PaymentSplitter} from "@openzeppelin/contracts/finance/PaymentSplitter.sol";
import {BitMaps} from "@openzeppelin/contracts/utils/structs/BitMaps.sol";

import {AbstractConsensus} from "../AbstractConsensus.sol";
import {IConsensus} from "../IConsensus.sol";
import {IHistory} from "../../history/IHistory.sol";

/// @title Quorum consensus
/// @notice A consensus model controlled by a small set of addresses, the validators.
///         In this version, the validator set is immutable.
/// @dev This contract uses OpenZeppelin `PaymentSplitter` and `BitMaps`.
///      For more information on those, please consult OpenZeppelin's official documentation.
contract Quorum is AbstractConsensus, PaymentSplitter {
    using BitMaps for BitMaps.BitMap;

    /// @notice The history contract.
    /// @dev See the `getHistory` function.
    IHistory internal immutable history;

    // Quorum members
    // Map an address to its index in the validator set.
    // The first validator has index 1. Thus, index 0 means the address is not in the validator set.
    mapping(address => uint256) public validatorIndex;
    uint256 public immutable quorumSize;

    // Quorum votes
    struct Voted {
        // how many has voted yea
        uint256 count;
        // use BitMap to record who has voted
        BitMaps.BitMap votedBitMap;
    }
    // Map a claim to struct Voted
    mapping(bytes => Voted) internal yeas;

    /// @notice Raised if not a validator
    error OnlyValidator();
    modifier onlyValidator() {
        if (validatorIndex[msg.sender] == 0) {
            revert OnlyValidator();
        }
        _;
    }

    /// @notice Construct a Quorum consensus
    /// @param _validators the list of validators
    /// @param _shares the list of shares
    /// @param _history the history contract
    /// @dev PaymentSplitter checks for duplicates in _validators
    constructor(
        address[] memory _validators,
        uint256[] memory _shares,
        IHistory _history
    ) PaymentSplitter(_validators, _shares) {
        // Add the array of validators into the quorum
        for (uint256 i; i < _validators.length; ++i) {
            validatorIndex[_validators[i]] = i + 1; // index starts from 1
        }
        quorumSize = _validators.length;
        history = _history;
    }

    /// @notice Vote for a claim to be submitted.
    ///         If this is the claim that reaches the majority, then
    ///         the claim is submitted to the history contract.
    ///         The encoding of `_claimData` might vary depending on the
    ///         implementation of the current history contract.
    /// @param _claimData Data for submitting a claim
    /// @dev Can only be called by a validator,
    ///      and the `Quorum` contract must have ownership over
    ///      its current history contract.
    function submitClaim(bytes calldata _claimData) external onlyValidator {
        Voted storage claimYeas = yeas[_claimData];
        uint256 index = validatorIndex[msg.sender];

        // If the msg.sender hasn't submitted the same claim before
        if (!claimYeas.votedBitMap.get(index)) {
            claimYeas.votedBitMap.set(index);

            // If this claim has now just over half of the quorum's votes,
            // then we can submit it to the history contract.
            if (++claimYeas.count == 1 + quorumSize / 2) {
                history.submitClaim(_claimData);
            }
        }
    }

    /// @notice Get the history contract.
    /// @return The history contract
    function getHistory() external view returns (IHistory) {
        return history;
    }

    /// @notice Get a claim from the current history.
    ///         The encoding of `_proofContext` might vary depending on the
    ///         implementation of the current history contract.
    /// @inheritdoc IConsensus
    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        return history.getClaim(_dapp, _proofContext);
    }
}
