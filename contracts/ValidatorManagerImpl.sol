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

interface ValidatorManagerImpl {
    address immutable descartesV2; // descartes 2 contract using this validator
    bytes32[] claims; // current's epoch claims
    address[] validators; // current validators
    bytes20 currentMask; // mask of validatos that agree on the current claims
    bytes20 consensusMask; // mask of all validators - cant be immutable because
                           // because validators can be added/kicked out


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
        newConsensusMask();
    }

    // @notice called when a claim is received by descartesv2
    // @params _sender address of sender of that claim
    // @params _claim claim received by descartesv2
    // @returns result of claim, signaling current state of claims
    function onClaim(
        address _sender,
        bytes32 _claim
    )
    onlyDescartesV2
    returns (Result)
    {
        // TODO: should claims by non validators just revert?
        if (!isAllowed(_sender)) return Result.NoConflict;

        claims.push(_claim);

        return claims[0] == _claim ?
            updateMask(_sender) :
            Result.Conflict;
    }

    // @notice called when a dispute ends in descartesv2
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function onDisputeEnd(
        address _winner,
        address _loser
    )
    onlyDescartesV2
    returns (Result) {
        // remove validator also updates consensus mask
        removeValidator(_loser);
        return updateMask(_winner);
    }

    // @notice called when a new epoch starts
    function onNewEpoch() onlyDescartesV2 {
        // clear claims array
        delete claims;

        // resets current mask
        currentMask = 0;
    }


    // @notice removes claim from claims[]
    // @returns claim being removed, returns 0x if there are no claims
    function popClaim() onlyDescartesV2 returns (bytes32) {
        uint cLength = claims.length;

        if (cLength == 0) return bytes32(0);

        bytes32 claim = claims[cLength - 1];
        claims.pop()

        return claim;
    }

    // INTERNAL FUNCTIONS

    // @notice creates a new consensus mask
    function newConsensusMask(){
        for (uint i = 0; i < validators.length; i++) {
            consensusMask = consensusMask & validators[i];
        }
    }

    // @notice updates current consensus mask
    // @params _sender address that will be included in mask
    // @return if mask updated led to consensus
    function updateMask(address _sender) internal returns (Result) {
        currentMask = currentMask & _sender;
        return currentMask == consensusMask ?
            Result.Consensus :
            Result.NoConflict;
    }

    // @notice removes a validator
    // @params address of validator to be removed
    function removeValidator(address _validator) internal {
        // finds and remove validator
        for (uint i = 0; i < validators.length; i++) {
            if (_validator == validators[i]) {
                validators[i] = validators[validators.length - 1];
                validators.pop();
                break;
            }
        }
        // creates new consensus mask
        newConsensusMask();
    }
}
