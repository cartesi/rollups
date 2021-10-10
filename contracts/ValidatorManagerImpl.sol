// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager Implementation
pragma solidity ^0.8.0;

import "./ValidatorManager.sol";

contract ValidatorManagerImpl is ValidatorManager {
    address immutable rollups; // rollups contract using this validator
    bytes32 currentClaim; // current claim - first claim of this epoch
    address payable[] validators; // current validators

    // A bit set for each validator that agrees with current claim,
    // on their respective positions
    uint32 claimAgreementMask;

    // Every validator who should approve (in order to reach consensus) will have a one set on this mask
    // This mask is updated if a validator is added or removed
    uint32 consensusGoalMask;

    // @notice functions modified by onlyRollups will only be executed if
    // they're called by Rollups contract, otherwise it will throw an exception
    function onlyRollups() internal view {
        require(msg.sender == rollups, "Only rollups");
    }

    // @notice populates validators array and creates a consensus mask
    // @params _rollups address of rollupscontract
    // @params _validators initial validator set
    // @dev validators have to be unique, if the same validator is added twice
    //      consensus will never be reached
    constructor(address _rollups, address payable[] memory _validators) {
        rollups = _rollups;
        validators = _validators;

        // create consensus goal, represents the scenario where all
        // all validators claimed and agreed
        consensusGoalMask = updateConsensusGoalMask();
    }

    // @notice called when a claim is received by rollups
    // @params _sender address of sender of that claim
    // @params _claim claim received by rollups
    // @return result of claim, Consensus | NoConflict | Conflict
    // @return [currentClaim, conflicting claim] if there is Conflict
    //         [currentClaim, bytes32(0)] if there is Consensus
    //         [bytes32(0), bytes32(0)] if there is NoConflcit
    // @return [claimer1, claimer2] if there is  Conflcit
    //         [claimer1, address(0)] if there is Consensus
    //         [address(0), address(0)] if there is NoConflcit
    function onClaim(address payable _sender, bytes32 _claim)
        public
        override
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        onlyRollups();
        require(_claim != bytes32(0), "empty claim");
        require(isAllowed(_sender), "sender not allowed");

        // cant return because a single claim might mean consensus
        if (currentClaim == bytes32(0)) {
            currentClaim = _claim;
        }

        if (_claim != currentClaim) {
            return
                emitClaimReceivedAndReturn(
                    Result.Conflict,
                    [currentClaim, _claim],
                    [getClaimerOfCurrentClaim(), _sender]
                );
        }
        claimAgreementMask = updateClaimAgreementMask(_sender);

        return
            isConsensus(claimAgreementMask, consensusGoalMask)
                ? emitClaimReceivedAndReturn(
                    Result.Consensus,
                    [_claim, bytes32(0)],
                    [_sender, payable(0)]
                )
                : emitClaimReceivedAndReturn(
                    Result.NoConflict,
                    [bytes32(0), bytes32(0)],
                    [payable(0), payable(0)]
                );
    }

    // @notice called when a dispute ends in rollups
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function onDisputeEnd(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    )
        public
        override
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        onlyRollups();

        // remove validator also removes validator from both bitmask
        removeFromValidatorSetAndBothBitmasks(_loser);

        if (_winningClaim == currentClaim) {
            // first claim stood, dont need to update the bitmask
            return
                isConsensus(claimAgreementMask, consensusGoalMask)
                    ? emitDisputeEndedAndReturn(
                        Result.Consensus,
                        [_winningClaim, bytes32(0)],
                        [_winner, payable(0)]
                    )
                    : emitDisputeEndedAndReturn(
                        Result.NoConflict,
                        [bytes32(0), bytes32(0)],
                        [payable(0), payable(0)]
                    );
        }

        // if first claim lost, and other validators have agreed with it
        // there is a new dispute to be played
        if (claimAgreementMask != 0) {
            return
                emitDisputeEndedAndReturn(
                    Result.Conflict,
                    [currentClaim, _winningClaim],
                    [getClaimerOfCurrentClaim(), _winner]
                );
        }
        // else there are no valdiators that agree with losing claim
        // we can update current claim and check for consensus in case
        // the winner is the only validator left
        currentClaim = _winningClaim;
        claimAgreementMask = updateClaimAgreementMask(_winner);
        return
            isConsensus(claimAgreementMask, consensusGoalMask)
                ? emitDisputeEndedAndReturn(
                    Result.Consensus,
                    [_winningClaim, bytes32(0)],
                    [_winner, payable(0)]
                )
                : emitDisputeEndedAndReturn(
                    Result.NoConflict,
                    [bytes32(0), bytes32(0)],
                    [payable(0), payable(0)]
                );
    }

    // @notice called when a new epoch starts
    // @return current claim
    function onNewEpoch() public override returns (bytes32) {
        onlyRollups();

        bytes32 tmpClaim = currentClaim;

        // clear current claim
        currentClaim = bytes32(0);
        // clear validator agreement bit mask
        claimAgreementMask = 0;

        emit NewEpoch(tmpClaim);
        return tmpClaim;
    }

    // @notice get agreement mask
    // @return current state of agreement mask
    function getCurrentAgreementMask() public view returns (uint32) {
        return claimAgreementMask;
    }

    // @notice get consensus goal mask
    // @return current consensus goal mask
    function getConsensusGoalMask() public view returns (uint32) {
        return consensusGoalMask;
    }

    // @notice get current claim
    // @return current claim
    function getCurrentClaim() public view override returns (bytes32) {
        return currentClaim;
    }

    // INTERNAL FUNCTIONS

    // @notice emits dispute ended event and then return
    // @param _result to be emitted and returned
    // @param _claims to be emitted and returned
    // @param _validators to be emitted and returned
    // @dev this function existis to make code more clear/concise
    function emitDisputeEndedAndReturn(
        Result _result,
        bytes32[2] memory _claims,
        address payable[2] memory _validators
    )
        internal
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        emit DisputeEnded(_result, _claims, _validators);
        return (_result, _claims, _validators);
    }

    // @notice emits claim received event and then return
    // @param _result to be emitted and returned
    // @param _claims to be emitted and returned
    // @param _validators to be emitted and returned
    // @dev this function existis to make code more clear/concise
    function emitClaimReceivedAndReturn(
        Result _result,
        bytes32[2] memory _claims,
        address payable[2] memory _validators
    )
        internal
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        emit ClaimReceived(_result, _claims, _validators);
        return (_result, _claims, _validators);
    }

    // @notice get one of the validators that agreed with current claim
    // @return validator that agreed with current claim
    function getClaimerOfCurrentClaim()
        internal
        view
        returns (address payable)
    {
        // TODO: we are always getting the first validator
        // on the array that agrees with the current claim to enter a dispute
        // should this be random?
        for (uint256 i; i < validators.length; i++) {
            if (claimAgreementMask & (1 << i) != 0) {
                return validators[i];
            }
        }
        revert("Agreeing validator not found");
    }

    // @notice updates the consensus goal mask
    // @return new consensus goal mask
    function updateConsensusGoalMask() internal view returns (uint32) {
        // consensus goal is a number where
        // all bits related to validators are turned on
        uint256 consensusMask = (1 << validators.length) - 1;
        return uint32(consensusMask);
    }

    // @notice updates mask of validators that agreed with current claim
    // @params _sender address that of validator that will be included in mask
    // @return new claim agreement mask
    function updateClaimAgreementMask(address payable _sender)
        internal
        view
        returns (uint32)
    {
        uint256 tmpClaimAgreement = claimAgreementMask;
        for (uint256 i; i < validators.length; i++) {
            if (_sender == validators[i]) {
                tmpClaimAgreement = (tmpClaimAgreement | (1 << i));
                break;
            }
        }

        return uint32(tmpClaimAgreement);
    }

    // @notice removes a validator
    // @params address of validator to be removed
    // @returns new claim agreement bitmask
    // @returns new consensus goal bitmask
    function removeFromValidatorSetAndBothBitmasks(address _validator)
        internal
    {
        // put address(0) in validators position
        // removes validator from claim agreement bitmask
        // removes validator from consensus goal mask
        for (uint256 i; i < validators.length; i++) {
            if (_validator == validators[i]) {
                validators[i] = payable(0);
                uint32 zeroMask = ~(uint32(1) << uint32(i));
                claimAgreementMask = claimAgreementMask & zeroMask;
                consensusGoalMask = consensusGoalMask & zeroMask;
                break;
            }
        }
    }

    function isAllowed(address _sender) internal view returns (bool) {
        for (uint256 i; i < validators.length; i++) {
            if (_sender == validators[i]) return true;
        }
        return false;
    }

    function isConsensus(
        uint256 _claimAgreementMask,
        uint256 _consensusGoalMask
    ) internal pure returns (bool) {
        return _claimAgreementMask == _consensusGoalMask;
    }
}
