// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager library (alternative version)
pragma solidity ^0.8.0;

import {Result} from "../../interfaces/IValidatorManager.sol";

// TODO: this libray seems to be very unsafe, need to think about security implications
library LibValidatorManager1 {
    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("ValidatorManager.diamond.storage");

    struct DiamondStorage {
        bytes32 currentClaim; // current claim - first claim of this epoch
        address payable[] validators; // current validators
        // A bit set for each validator that agrees with current claim,
        // on their respective positions
        uint32 claimAgreementMask;
        // Every validator who should approve (in order to reach consensus) will have a one set on this mask
        // This mask is updated if a validator is added or removed
        uint32 consensusGoalMask;
    }

    /// @notice emitted on Claim received
    event ClaimReceived(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on Dispute end
    event DisputeEnded(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on new Epoch
    event NewEpoch(bytes32 claim);

    function diamondStorage()
        internal
        pure
        returns (DiamondStorage storage ds)
    {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    /// @notice called when a dispute ends in rollups
    /// @param ds diamond storage pointer
    /// @param winner address of dispute winner
    /// @param loser address of dispute loser
    /// @param winningClaim the winning claim
    /// @return result of dispute being finished
    function onDisputeEnd(
        DiamondStorage storage ds,
        address payable winner,
        address payable loser,
        bytes32 winningClaim
    ) internal returns (Result, bytes32[2] memory, address payable[2] memory) {
        // remove validator also removes validator from both bitmask
        removeFromValidatorSetAndBothBitmasks(ds, loser);

        if (winningClaim == ds.currentClaim) {
            // first claim stood, dont need to update the bitmask
            return
                isConsensus(ds.claimAgreementMask, ds.consensusGoalMask)
                    ? emitDisputeEndedAndReturn(
                        Result.Consensus,
                        [winningClaim, bytes32(0)],
                        [winner, payable(0)]
                    )
                    : emitDisputeEndedAndReturn(
                        Result.NoConflict,
                        [winningClaim, bytes32(0)],
                        [winner, payable(0)]
                    );
        }

        // if first claim lost, and other validators have agreed with it
        // there is a new dispute to be played
        if (ds.claimAgreementMask != 0) {
            return
                emitDisputeEndedAndReturn(
                    Result.Conflict,
                    [ds.currentClaim, winningClaim],
                    [getClaimerOfCurrentClaim(ds), winner]
                );
        }
        // else there are no valdiators that agree with losing claim
        // we can update current claim and check for consensus in case
        // the winner is the only validator left
        ds.currentClaim = winningClaim;
        ds.claimAgreementMask = updateClaimAgreementMask(ds, winner);
        return
            isConsensus(ds.claimAgreementMask, ds.consensusGoalMask)
                ? emitDisputeEndedAndReturn(
                    Result.Consensus,
                    [winningClaim, bytes32(0)],
                    [winner, payable(0)]
                )
                : emitDisputeEndedAndReturn(
                    Result.NoConflict,
                    [winningClaim, bytes32(0)],
                    [winner, payable(0)]
                );
    }

    /// @notice called when a new epoch starts
    /// @param ds diamond storage pointer
    /// @return current claim
    function onNewEpoch(DiamondStorage storage ds) internal returns (bytes32) {
        bytes32 tmpClaim = ds.currentClaim;

        // clear current claim
        ds.currentClaim = bytes32(0);
        // clear validator agreement bit mask
        ds.claimAgreementMask = 0;

        emit NewEpoch(tmpClaim);
        return tmpClaim;
    }

    /// @notice called when a claim is received by rollups
    /// @param ds diamond storage pointer
    /// @param sender address of sender of that claim
    /// @param claim claim received by rollups
    /// @return result of claim, Consensus | NoConflict | Conflict
    /// @return [currentClaim, conflicting claim] if there is Conflict
    ///         [currentClaim, bytes32(0)] if there is Consensus or NoConflcit
    /// @return [claimer1, claimer2] if there is  Conflcit
    ///         [claimer1, address(0)] if there is Consensus or NoConflcit
    function onClaim(
        DiamondStorage storage ds,
        address payable sender,
        bytes32 claim
    ) internal returns (Result, bytes32[2] memory, address payable[2] memory) {
        require(claim != bytes32(0), "empty claim");
        require(isValidator(ds, sender), "sender not allowed");

        // cant return because a single claim might mean consensus
        if (ds.currentClaim == bytes32(0)) {
            ds.currentClaim = claim;
        }

        if (claim != ds.currentClaim) {
            return
                emitClaimReceivedAndReturn(
                    Result.Conflict,
                    [ds.currentClaim, claim],
                    [getClaimerOfCurrentClaim(ds), sender]
                );
        }
        ds.claimAgreementMask = updateClaimAgreementMask(ds, sender);

        return
            isConsensus(ds.claimAgreementMask, ds.consensusGoalMask)
                ? emitClaimReceivedAndReturn(
                    Result.Consensus,
                    [claim, bytes32(0)],
                    [sender, payable(0)]
                )
                : emitClaimReceivedAndReturn(
                    Result.NoConflict,
                    [claim, bytes32(0)],
                    [sender, payable(0)]
                );
    }

    /// @notice emits dispute ended event and then return
    /// @param result to be emitted and returned
    /// @param claims to be emitted and returned
    /// @param validators to be emitted and returned
    /// @dev this function existis to make code more clear/concise
    function emitDisputeEndedAndReturn(
        Result result,
        bytes32[2] memory claims,
        address payable[2] memory validators
    ) internal returns (Result, bytes32[2] memory, address payable[2] memory) {
        emit DisputeEnded(result, claims, validators);
        return (result, claims, validators);
    }

    /// @notice emits claim received event and then return
    /// @param result to be emitted and returned
    /// @param claims to be emitted and returned
    /// @param validators to be emitted and returned
    /// @dev this function existis to make code more clear/concise
    function emitClaimReceivedAndReturn(
        Result result,
        bytes32[2] memory claims,
        address payable[2] memory validators
    ) internal returns (Result, bytes32[2] memory, address payable[2] memory) {
        emit ClaimReceived(result, claims, validators);
        return (result, claims, validators);
    }

    /// @notice get one of the validators that agreed with current claim
    /// @param ds diamond storage pointer
    /// @return validator that agreed with current claim
    function getClaimerOfCurrentClaim(
        DiamondStorage storage ds
    ) internal view returns (address payable) {
        // TODO: we are always getting the first validator
        // on the array that agrees with the current claim to enter a dispute
        // should this be random?
        for (uint256 i; i < ds.validators.length; i++) {
            if (ds.claimAgreementMask & (1 << i) != 0) {
                return ds.validators[i];
            }
        }
        revert("Agreeing validator not found");
    }

    /// @notice updates the consensus goal mask
    /// @param ds diamond storage pointer
    /// @return new consensus goal mask
    function updateConsensusGoalMask(
        DiamondStorage storage ds
    ) internal view returns (uint32) {
        // consensus goal is a number where
        // all bits related to validators are turned on
        uint256 consensusMask = (1 << ds.validators.length) - 1;
        return uint32(consensusMask);
    }

    /// @notice updates mask of validators that agreed with current claim
    /// @param ds diamond storage pointer
    /// @param sender address that of validator that will be included in mask
    /// @return new claim agreement mask
    function updateClaimAgreementMask(
        DiamondStorage storage ds,
        address payable sender
    ) internal view returns (uint32) {
        uint256 tmpClaimAgreement = ds.claimAgreementMask;
        for (uint256 i; i < ds.validators.length; i++) {
            if (sender == ds.validators[i]) {
                tmpClaimAgreement = (tmpClaimAgreement | (1 << i));
                break;
            }
        }

        return uint32(tmpClaimAgreement);
    }

    /// @notice removes a validator
    /// @param ds diamond storage pointer
    /// @param validator address of validator to be removed
    function removeFromValidatorSetAndBothBitmasks(
        DiamondStorage storage ds,
        address validator
    ) internal {
        // put address(0) in validators position
        // removes validator from claim agreement bitmask
        // removes validator from consensus goal mask
        for (uint256 i; i < ds.validators.length; i++) {
            if (validator == ds.validators[i]) {
                ds.validators[i] = payable(0);
                uint32 zeroMask = ~(uint32(1) << uint32(i));
                ds.claimAgreementMask = ds.claimAgreementMask & zeroMask;
                ds.consensusGoalMask = ds.consensusGoalMask & zeroMask;
                break;
            }
        }
    }

    function isValidator(
        DiamondStorage storage ds,
        address sender
    ) internal view returns (bool) {
        for (uint256 i; i < ds.validators.length; i++) {
            if (sender == ds.validators[i]) return true;
        }

        return false;
    }

    function isConsensus(
        uint256 claimAgreementMask,
        uint256 consensusGoalMask
    ) internal pure returns (bool) {
        return claimAgreementMask == consensusGoalMask;
    }
}
