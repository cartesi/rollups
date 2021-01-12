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


/// @title Validator Manager
pragma solidity ^0.7.0;

// TODO: What is the incentive for validators to not just copy the first claim that arrived?
contract ValidatorManager {
    uint256 collateral; // required collateral per validator
    uint8 maxSize; // max amount of validators accepted
    uint8 minParticipationRate; // forced num of participations (claim) on the last 100 blocks?
    address[] validators;
    address[] pendingValidators;

    struct Validator {
        address validator;
        uint256[] epochsWithClaim; // epochs in which this validator sent a claim TODO: need a better name for this
        uint8 participationRate; // number of claims on the last x blocks
    }

    // TODO: Add Events

    // returns true if registration was successful
    function addValidators(address[] _validators) onlyOwner returns bool {
        // creates list of validators
        // add to pending validators, because collateral was not yet deposited
    }

    // can only be called by pending validator
    // returns true if validator was accepted
    function acceptValidatorRole()
    onlyPendingValidator
    returns (bool) {
        // msg.sender transfers collateral to this contract
        // msg.sender goes from pending validators to validators list
    }


    // returns if registrantion withdraw was successful
    function retire() onlyValidators returns bool {
        // cant do this if last claim/challenge of msg.sender is still active
        // transfer back collateral to validator
        // remove from validators list
    }

    // returns if all validators on _validators list were removed successfully
    function removeValidators(address[] _validators, bytes32[] fraudProof) returns bool {
        // prove that a validator cheated (can you only remove when they cheated? Can you remove from inactivity?)
        // maybe you can send either a fraudProof or a list of the x most recent epochs where this validator hasn't participated
        // but then the lazy validator can participate on the minimum amount of epochs possible, just to not get kicked
        // not sure if this would make economical sense, because his collateral would be locked and he wouldn't be gettint the retainer fee. But it might be a griefing attack of sorts.


        // takes collateral
        // kick validator
    }

    // TODO: Only CTSI? Only Ether? Any ERC20 token?
    function transferRetainer(uint256 _epoch) onlyCartesiContract returns (bool) {
        // pay retainer to validators who contributed
    }

    // GETTERS

    // returns is address is an active validator
    function isValidator(address _validator) returns bool{}

    // returns if validator has claimed for epoch _epoch
    // returns validator claim
    function hasClaimed(address _validator, uint256 _epoch)
    returns (bool, bytes32) {}

    // returns set of active validators
    function getCurrentValidatorSet() returns address[] {}

    //returns validator set that claimed for epoch _epoch
    function getValidatorSetOnEpoch() returns address[] {}

    // returns all claims by validator
    function getClaimHistory(address _validator) returns bytes32[] {}

    // returns epochs in which validator got into a challenge
    function getContestedEpochs(address _validator) returns uint256[] {}

    // returns collateral
    function getCollateral() returns uint256 {}

    // returns contract's balance
    function getBalance() returns uint256 {}
}
