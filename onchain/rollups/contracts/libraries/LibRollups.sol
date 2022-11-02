// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups library
pragma solidity ^0.8.0;

import {Phase} from "../interfaces/IRollups.sol";
import {Result} from "../interfaces/IValidatorManager.sol";

import {LibInput} from "../libraries/LibInput.sol";
import {LibOutput} from "../libraries/LibOutput.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";
import {LibDisputeManager} from "../libraries/LibDisputeManager.sol";

library LibRollups {
    using LibInput for LibInput.DiamondStorage;
    using LibOutput for LibOutput.DiamondStorage;
    using LibValidatorManager for LibValidatorManager.DiamondStorage;

    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("Rollups.diamond.storage");

    struct DiamondStorage {
        bytes32 templateHash; // state hash of the cartesi machine at t0
        uint32 inputDuration; // duration of input accumulation phase in seconds
        uint32 challengePeriod; // duration of challenge period in seconds
        uint32 inputAccumulationStart; // timestamp when current input accumulation phase started
        uint32 sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable
        uint32 currentPhase_int; // current phase in integer form
    }

    /// @notice epoch finalized
    /// @param epochNumber number of the epoch being finalized
    /// @param epochHash claim being submitted by this epoch
    event FinalizeEpoch(uint256 indexed epochNumber, bytes32 epochHash);

    /// @notice dispute resolved
    /// @param winner winner of dispute
    /// @param loser loser of dispute
    /// @param winningClaim initial claim of winning validator
    event ResolveDispute(address winner, address loser, bytes32 winningClaim);

    /// @notice phase change
    /// @param newPhase new phase
    event PhaseChange(Phase newPhase);

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

    /// @notice called when new input arrives, manages the phase changes
    /// @param ds diamond storage pointer
    /// @dev can only be called by input contract
    function notifyInput(DiamondStorage storage ds) internal returns (bool) {
        Phase currentPhase = Phase(ds.currentPhase_int);
        uint256 inputAccumulationStart = ds.inputAccumulationStart;
        uint256 inputDuration = ds.inputDuration;

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            ds.currentPhase_int = uint32(Phase.AwaitingConsensus);
            emit PhaseChange(Phase.AwaitingConsensus);
            return true;
        }
        return false;
    }

    /// @notice called when a dispute is resolved by the dispute manager
    /// @param ds diamond storage pointer
    /// @param winner winner of dispute
    /// @param loser loser of dispute
    /// @param winningClaim initial claim of winning validator
    function resolveDispute(
        DiamondStorage storage ds,
        address payable winner,
        address payable loser,
        bytes32 winningClaim
    ) internal {
        Result result;
        bytes32[2] memory claims;
        address payable[2] memory claimers;
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();

        (result, claims, claimers) = validatorManagerDS.onDisputeEnd(
            winner,
            loser,
            winningClaim
        );

        // restart challenge period
        ds.sealingEpochTimestamp = uint32(block.timestamp);

        emit ResolveDispute(winner, loser, winningClaim);
        resolveValidatorResult(ds, result, claims, claimers);
    }

    /// @notice resolve results returned by validator manager
    /// @param ds diamond storage pointer
    /// @param result result from claim or dispute operation
    /// @param claims array of claims in case of new conflict
    /// @param claimers array of claimers in case of new conflict
    function resolveValidatorResult(
        DiamondStorage storage ds,
        Result result,
        bytes32[2] memory claims,
        address payable[2] memory claimers
    ) internal {
        if (result == Result.NoConflict) {
            Phase currentPhase = Phase(ds.currentPhase_int);
            if (currentPhase != Phase.AwaitingConsensus) {
                ds.currentPhase_int = uint32(Phase.AwaitingConsensus);
                emit PhaseChange(Phase.AwaitingConsensus);
            }
        } else if (result == Result.Consensus) {
            startNewEpoch(ds);
        } else {
            // for the case when result == Result.Conflict
            Phase currentPhase = Phase(ds.currentPhase_int);
            if (currentPhase != Phase.AwaitingDispute) {
                ds.currentPhase_int = uint32(Phase.AwaitingDispute);
                emit PhaseChange(Phase.AwaitingDispute);
            }
            LibDisputeManager.initiateDispute(claims, claimers);
        }
    }

    /// @notice starts new epoch
    /// @param ds diamond storage pointer
    function startNewEpoch(DiamondStorage storage ds) internal {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();

        // reset input accumulation start and deactivate challenge period start
        ds.currentPhase_int = uint32(Phase.InputAccumulation);
        emit PhaseChange(Phase.InputAccumulation);
        ds.inputAccumulationStart = uint32(block.timestamp);
        ds.sealingEpochTimestamp = type(uint32).max;

        bytes32 finalClaim = validatorManagerDS.onNewEpoch();

        // emit event before finalized epoch is added to the Output storage
        emit FinalizeEpoch(outputDS.getNumberOfFinalizedEpochs(), finalClaim);

        outputDS.onNewEpoch(finalClaim);
        inputDS.onNewEpoch();
    }

    /// @notice returns index of current (accumulating) epoch
    /// @param ds diamond storage pointer
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    ///      of finalized epochs array, else there are two non finalized epochs,
    ///      one awaiting consensus/dispute and another accumulating input
    function getCurrentEpoch(
        DiamondStorage storage ds
    ) internal view returns (uint256) {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();

        uint256 finalizedEpochs = outputDS.getNumberOfFinalizedEpochs();

        Phase currentPhase = Phase(ds.currentPhase_int);

        return
            currentPhase == Phase.InputAccumulation
                ? finalizedEpochs
                : finalizedEpochs + 1;
    }
}
