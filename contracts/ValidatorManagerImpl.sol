// Copyright (C) 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: GPL-3.0-only
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.


/// @title Validator Manager Implementation
pragma solidity ^0.7.0;

import "./ValidatorManager.sol"

contract ValidatorManagerImpl is ValidatorManager {
    address immutable descartesV2; // descartes 2 contract using this validator
    bytes32 claim; // first claim
    address[] validators; // current validators
    uint32 validatorBitmask; //  each validator is represented by one bit
    uint32 consensusGoal; // number that represents validatorBitMask with all relevant bits turned on


    // @notice functions modified by onlyDescartesV2 will only be executed if
    // they're called by DescartesV2 contract, otherwise it will throw an exception
    modifier onlyDescartesV2 {
        require(
            msg.sender == descartesV2,
            "Only descartesV2 can call this functions"
        );
        _;
    }

    // @notice populates validators array and creates a consensus mask
    // @params _descartesV2 address of descartes contract
    // @params _validators initial validator set
    constructor(address _descartesV2, address[] _validators) {
        descartesV2 = _descartesV2;
        validators = _validators;

        // create consensus goal, represents the scenario where all
        // all validators claimed and agreed
        newConsensusGoal();
    }

    // @notice called when a claim is received by descartesv2
    // @params _sender address of sender of that claim
    // @params _claim claim received by descartesv2
    // @returns result of claim, signaling current state of claims
    function onClaim(
        address _sender,
        bytes32 _claim
    )
    public
    onlyDescartesV2
    returns (Result, bytes32[2] claims, address[2] claimers)
    {
        // TODO: should claims by non validators just revert?
        if (!isAllowed(_sender)) return (Result.NoConflict, new bytes32[2], new address[2];

        if (claim == bytes32(0)) {
            claim = _claim;
        }

        if (_claim != claim) {
            return (
                Result.Conflict,
                bytes32[claim, _claim],
                address[popClaimer(), _sender]
            );
        }

        return (
            updateValidatorMask(_sender),
            new bytes32[_claim, bytes32(0)],
            new address[_sender, address(0)]
        );
    }

    // @notice called when a dispute ends in descartesv2
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function onDisputeEnd(
        address _winner,
        address _loser,
        bytes32 _winningClaim
    )
    onlyDescartesV2
    public
    returns (Result, bytes32[2], address[2]) {
        // remove validator also removes validator from bitmask
        removeValidator(_loser);

        newConsensusGoal();
        if (_winningClaim == claim) {
            // first claim stood, dont need to update the bitmask
            return consensusGoal == validatorBitMask ?
                (Result.Consensus, bytes32[_winningClaim, bytes32(0)], address[_winner, address(0)]) :
                (Result.NoConflict, bytes32[_winningClaim, bytes32(0)], address[_winner, address(0)]); 
        }

        // if first claim lost, and other validators have agreed with it
        // there is a new dispute to be played
        if (validatorBitMask != 0) {
            return (
                Result.Conflict,
                bytes32[claim, _winningClaim],
                address[popClaimer(), _winner]
            );
        }
        // else there are no valdiators that agree with losing claim
        return updateValidatorMask(_winner);
    }

    // @notice called when a new epoch starts
    function onNewEpoch() public onlyDescartesV2 {
        claim = bytes32(0);
        validatorBitMask = 0;
    }

    // INTERNAL FUNCTIONS

    // @notice get validator that claimed current claim
    function popClaimer() internal returns (address) {
        require(
            validatorBitMask == 0,
             "No validators agree with current claim"
        );

        for (uint i = 0; i < validators.length(); i++) {
            if (validatorBitMask & (1 << i) == 1) {
               return validators[i];
            }
        }
    }

    // @notice creates a new consensus goal
    function newConsensusGoal() internal {
        // consensus goal is a number where
        // all bits related to validators are turned on
        uint32 tmpConsensusGoal = (2 ** validators.length) - 1;

        // is it cheaper to start from zero and update for non zero addresses?
        for (uint i = 0; i < validators.length; i++) {
            if (validators[i] == address(0)) {
                uint32 zeroMask = ~(1 << i);
                tmpConsensusGoal = tmpConsensusGoal & zeroMask;
            }
        }
        consensusGoal = tempoConsensusGoal;
    }

    // @notice updates current validator bit mask
    // @params _sender address that will be included in mask
    // @return if mask updated led to consensus
    function updateValidatorMask(address _sender) internal returns (Result) {
        for (uint i = 0; i < validators.length; i++) {
            if (_sender == validators[i]) break;
        }
        validatorBitMask = validatorBitMask & (1 << i);

        return validatorBitMask == consensusGoal? Result.Consensus : Result.NoConflict;
    }

    // @notice removes a validator
    // @params address of validator to be removed
    function removeValidator(address _validator) internal {
        // put address(0) in validators position
        // removes them from validator bitmask
        for (uint i = 0; i < validators.length; i++) {
            if (_validator == validators[i]) {
                validators[i] = address(0);
                bytes32 zeroMask = ~(1 << i);
                validatorsBitMask = validatorBitMask & zeroMask;
                break;
            }
        }
    }
}
