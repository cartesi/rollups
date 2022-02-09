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
pragma solidity >=0.8.8;

import "./ValidatorManager.sol";
import "./ClaimsMaskLibrary.sol";

contract ValidatorManagerClaimsCountedImpl is ValidatorManager {
    address immutable rollups; // rollups contract using this validator
    bytes32 currentClaim; // current claim - first claim of this epoch
    address payable[] public validators; // up to 8 validators
    uint256 public maxNumValidators; // the maximum number of validators, set in the constructor

    // A bit set used for up to 8 validators.
    // The first 8 bits are used to indicate whom supports the current claim
    // The second 8 bits are used to indicate those should have claimed in order to reach consensus
    // The following every 30 bits are used to indicate the number of total claims each validator has made
    // | agreement mask | consensus mask | #claims_validator7 | #claims_validator6 | ... | #claims_validator0 |
    // |     8 bits     |     8 bits     |      30 bits       |      30 bits       | ... |      30 bits       |
    ClaimsMask claimsMask;
    using ClaimsMaskLibrary for ClaimsMask;

    /// @notice functions modified by onlyRollups will only be executed if
    ///         they're called by Rollups contract, otherwise it will throw an exception
    function onlyRollups() internal view {
        require(msg.sender == rollups, "Only rollups");
    }

    /// @notice populates validators array and creates a consensus mask
    /// @param _rollups address of rollupscontract
    /// @param _validators initial validator set
    /// @dev validators have to be unique, if the same validator is added twice
    ///      consensus will never be reached
    constructor(address _rollups, address payable[] memory _validators) {
        rollups = _rollups;

        require(_validators.length <= 8, "up to 8 validators");
        validators = _validators;
        maxNumValidators = _validators.length;

        // create a new ClaimsMask, with only the consensus goal set,
        //      according to the number of validators
        claimsMask = ClaimsMaskLibrary.newClaimsMaskWithConsensusGoalSet(
            maxNumValidators
        );
    }

    /// @notice called when a claim is received by rollups
    /// @param _sender address of sender of that claim
    /// @param _claim claim received by rollups
    /// @return result of claim, Consensus | NoConflict | Conflict
    /// @return [currentClaim, conflicting claim] if there is Conflict
    ///         [currentClaim, bytes32(0)] if there is Consensus or NoConflcit
    /// @return [claimer1, claimer2] if there is  Conflcit
    ///         [claimer1, address(0)] if there is Consensus or NoConflcit
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
        require(isValidator(_sender), "sender not allowed");

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
        updateClaimAgreementMask(_sender);

        return
            isConsensus()
                ? emitClaimReceivedAndReturn(
                    Result.Consensus,
                    [_claim, bytes32(0)],
                    [_sender, payable(0)]
                )
                : emitClaimReceivedAndReturn(
                    Result.NoConflict,
                    [_claim, bytes32(0)],
                    [_sender, payable(0)]
                );
    }

    /// @notice called when a dispute ends in rollups
    /// @param _winner address of dispute winner
    /// @param _loser address of dispute loser
    /// @return result of dispute being finished
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

        removeValidator(_loser);

        if (_winningClaim == currentClaim) {
            // first claim stood, dont need to update the bitmask
            return
                isConsensus()
                    ? emitDisputeEndedAndReturn(
                        Result.Consensus,
                        [_winningClaim, bytes32(0)],
                        [_winner, payable(0)]
                    )
                    : emitDisputeEndedAndReturn(
                        Result.NoConflict,
                        [_winningClaim, bytes32(0)],
                        [_winner, payable(0)]
                    );
        }

        // if first claim lost, and other validators have agreed with it
        // there is a new dispute to be played
        if (claimsMask.getAgreementMask() != 0) {
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
        updateClaimAgreementMask(_winner);
        return
            isConsensus()
                ? emitDisputeEndedAndReturn(
                    Result.Consensus,
                    [_winningClaim, bytes32(0)],
                    [_winner, payable(0)]
                )
                : emitDisputeEndedAndReturn(
                    Result.NoConflict,
                    [_winningClaim, bytes32(0)],
                    [_winner, payable(0)]
                );
    }

    /// @notice called when a new epoch starts
    /// @return current claim
    function onNewEpoch() public override returns (bytes32) {
        onlyRollups();

        // reward validators who has made the correct claim by increasing their #claims
        claimFinalizedIncreaseCounts();

        bytes32 tmpClaim = currentClaim;

        // clear current claim
        currentClaim = bytes32(0);
        // clear validator agreement bit mask
        claimsMask = claimsMask.clearAgreementMask();

        emit NewEpoch(tmpClaim);
        return tmpClaim;
    }

    /// @notice get agreement mask
    /// @return current state of agreement mask
    function getAgreementMask() public view returns (uint256) {
        return claimsMask.getAgreementMask();
    }

    /// @notice get consensus goal mask
    /// @return current consensus goal mask
    function getConsensusGoalMask() public view returns (uint256) {
        return claimsMask.getConsensusGoalMask();
    }

    /// @notice get current claim
    /// @return current claim
    function getCurrentClaim() public view override returns (bytes32) {
        return currentClaim;
    }

    /// @notice get number of claims the sender has made
    /// @param _sender address
    /// @return #claims
    function getNumberOfClaimsByAddress(address payable _sender)
        public
        view
        returns (uint256)
    {
        for (uint256 i; i < validators.length; i++) {
            if (_sender == validators[i]) {
                return getNumberOfClaimsByIndex(i);
            }
        }
        // if validator not found
        return 0;
    }

    /// @notice find the validator and return the index or revert
    /// @param _sender address
    /// @return validator index or revert
    function getValidatorIndex(address _sender) public view returns (uint256) {
        require(_sender != address(0), "address 0");
        for (uint256 i; i < validators.length; i++) {
            if (_sender == validators[i]) return i;
        }
        revert("validator not found");
    }

    /// @notice get number of claims by the index in the validator set
    /// @param index the index in validator set
    /// @return #claims
    function getNumberOfClaimsByIndex(uint256 index)
        public
        view
        returns (uint256)
    {
        return claimsMask.getNumClaims(index);
    }

    // BELOW ARE INTERNAL FUNCTIONS

    // @notice only call this function when a claim has been finalized
    // Either a consensus has been reached or challenge period has past
    function claimFinalizedIncreaseCounts() internal {
        uint256 agreementMask = claimsMask.getAgreementMask();
        for (uint256 i; i < validators.length; i++) {
            // if a validator agrees with the current claim
            if ((agreementMask & (1 << i)) != 0) {
                // increase #claims by 1
                claimsMask = claimsMask.increaseNumClaims(i, 1);
            }
        }
    }

    /// @notice emits dispute ended event and then return
    /// @param _result to be emitted and returned
    /// @param _claims to be emitted and returned
    /// @param _validators to be emitted and returned
    /// @dev this function existis to make code more clear/concise
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

    /// @notice emits claim received event and then return
    /// @param _result to be emitted and returned
    /// @param _claims to be emitted and returned
    /// @param _validators to be emitted and returned
    /// @dev this function existis to make code more clear/concise
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

    /// @notice get one of the validators that agreed with current claim
    /// @return validator that agreed with current claim
    function getClaimerOfCurrentClaim()
        internal
        view
        returns (address payable)
    {
        // TODO: we are always getting the first validator
        // on the array that agrees with the current claim to enter a dispute
        // should this be random?
        uint256 agreementMask = claimsMask.getAgreementMask();
        for (uint256 i; i < validators.length; i++) {
            if (agreementMask & (1 << i) != 0) {
                return validators[i];
            }
        }
        revert("Agreeing validator not found");
    }

    /// @notice updates mask of validators that agreed with current claim
    /// @param _sender address of validator that will be included in mask
    function updateClaimAgreementMask(address payable _sender) internal {
        uint256 validatorIndex = getValidatorIndex(_sender);
        claimsMask = claimsMask.setAgreementMask(validatorIndex);
    }

    /// @notice removes a validator
    /// @param _validator address of validator to be removed
    function removeValidator(address _validator) internal {
        for (uint256 i; i < validators.length; i++) {
            if (_validator == validators[i]) {
                // put address(0) in validators position
                validators[i] = payable(0);
                // remove the validator from claimsMask
                claimsMask = claimsMask.removeValidator(i);
                break;
            }
        }
    }

    /// @notice check if the sender is a validator
    /// @param _sender sender address
    function isValidator(address _sender) internal view returns (bool) {
        require(_sender != address(0), "address 0");
        for (uint256 i; i < validators.length; i++) {
            if (_sender == validators[i]) return true;
        }
        return false;
    }

    /// @notice check if consensus has been reached
    function isConsensus() internal view returns (bool) {
        return
            claimsMask.getAgreementMask() == claimsMask.getConsensusGoalMask();
    }
}
