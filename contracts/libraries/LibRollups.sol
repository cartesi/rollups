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
    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("Rollups.diamond.storage");

    struct DiamondStorage {
        uint32 inputDuration; // duration of input accumulation phase in seconds
        uint32 challengePeriod; // duration of challenge period in seconds
        uint32 inputAccumulationStart; // timestamp when current input accumulation phase started
        uint32 sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable
        uint32 currentPhase_int; // current phase in integer form
    }

    /// @notice epoch finalized
    /// @param _epochNumber number of the epoch being finalized
    /// @param _epochHash claim being submitted by this epoch
    event FinalizeEpoch(uint256 indexed _epochNumber, bytes32 _epochHash);

    /// @notice dispute resolved
    /// @param _winner winner of dispute
    /// @param _loser loser of dispute
    /// @param _winningClaim initial claim of winning validator
    event ResolveDispute(
        address _winner,
        address _loser,
        bytes32 _winningClaim
    );

    /// @notice phase change
    /// @param _newPhase new phase
    event PhaseChange(Phase _newPhase);

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
    /// @dev can only be called by input contract
    function notifyInput() internal returns (bool) {
        DiamondStorage storage ds = diamondStorage();

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
    /// @param _winner winner of dispute
    /// @param _loser loser of dispute
    /// @param _winningClaim initial claim of winning validator
    function resolveDispute(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    ) internal {
        DiamondStorage storage ds = diamondStorage();

        Result result;
        bytes32[2] memory claims;
        address payable[2] memory claimers;

        (result, claims, claimers) = LibValidatorManager.onDisputeEnd(
            _winner,
            _loser,
            _winningClaim
        );

        // restart challenge period
        ds.sealingEpochTimestamp = uint32(block.timestamp);

        emit ResolveDispute(_winner, _loser, _winningClaim);
        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice resolve results returned by validator manager
    /// @param _result result from claim or dispute operation
    /// @param _claims array of claims in case of new conflict
    /// @param _claimers array of claimers in case of new conflict
    function resolveValidatorResult(
        Result _result,
        bytes32[2] memory _claims,
        address payable[2] memory _claimers
    ) internal {
        DiamondStorage storage ds = diamondStorage();

        if (_result == Result.NoConflict) {
            Phase currentPhase = Phase(ds.currentPhase_int);
            if (currentPhase != Phase.AwaitingConsensus) {
                ds.currentPhase_int = uint32(Phase.AwaitingConsensus);
                emit PhaseChange(Phase.AwaitingConsensus);
            }
        } else if (_result == Result.Consensus) {
            startNewEpoch();
        } else {
            // for the case when _result == Result.Conflict
            Phase currentPhase = Phase(ds.currentPhase_int);
            if (currentPhase != Phase.AwaitingDispute) {
                ds.currentPhase_int = uint32(Phase.AwaitingDispute);
                emit PhaseChange(Phase.AwaitingDispute);
            }
            LibDisputeManager.initiateDispute(_claims, _claimers);
        }
    }

    /// @notice starts new epoch
    function startNewEpoch() internal {
        DiamondStorage storage ds = diamondStorage();

        // reset input accumulation start and deactivate challenge period start
        ds.currentPhase_int = uint32(Phase.InputAccumulation);
        emit PhaseChange(Phase.InputAccumulation);
        ds.inputAccumulationStart = uint32(block.timestamp);
        ds.sealingEpochTimestamp = type(uint32).max;

        bytes32 finalClaim = LibValidatorManager.onNewEpoch();

        // emit event before finalized epoch is added to the Output storage
        emit FinalizeEpoch(LibOutput.getNumberOfFinalizedEpochs(), finalClaim);

        LibOutput.onNewEpoch(finalClaim);
        LibInput.onNewEpoch();
    }

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    //       of finalized epochs array, else there are two non finalized epochs,
    //       one awaiting consensus/dispute and another accumulating input
    function getCurrentEpoch() internal view returns (uint256) {
        DiamondStorage storage ds = diamondStorage();

        uint256 finalizedEpochs = LibOutput.getNumberOfFinalizedEpochs();

        Phase currentPhase = Phase(ds.currentPhase_int);

        return
            currentPhase == Phase.InputAccumulation
                ? finalizedEpochs
                : finalizedEpochs + 1;
    }
}
