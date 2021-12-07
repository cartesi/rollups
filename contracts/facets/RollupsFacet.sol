// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups facet
pragma solidity ^0.8.0;

import {IRollups, Phase} from "../interfaces/IRollups.sol";
import {Result} from "../interfaces/IValidatorManager.sol";

import {LibRollups} from "../libraries/LibRollups.sol";
import {LibInput} from "../libraries/LibInput.sol";
import {LibOutput} from "../libraries/LibOutput.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";

contract RollupsFacet is IRollups {
    ////
    //                             All claims agreed OR challenge period ended
    //                              functions: claim() or finalizeEpoch()
    //                        +--------------------------------------------------+
    //                        |                                                  |
    //               +--------v-----------+   new input after IPAD     +---------+----------+
    //               |                    +--------------------------->+                    |
    //   START  ---> | Input Accumulation |   firt claim after IPAD    | Awaiting Consensus |
    //               |                    +--------------------------->+                    |
    //               +-+------------------+                            +-----------------+--+
    //                 ^                                                                 ^  |
    //                 |                                              dispute resolved   |  |
    //                 |  dispute resolved                            before challenge   |  |
    //                 |  after challenge     +--------------------+  period ended       |  |
    //                 |  period ended        |                    +---------------------+  |
    //                 +----------------------+  Awaiting Dispute  |                        |
    //                                        |                    +<-----------------------+
    //                                        +--------------------+    conflicting claim
    ///

    using LibRollups for LibRollups.DiamondStorage;
    using LibInput for LibInput.DiamondStorage;
    using LibOutput for LibOutput.DiamondStorage;
    using LibValidatorManager for LibValidatorManager.DiamondStorage;

    /// @notice claim the result of current epoch
    /// @param _epochHash hash of epoch
    /// @dev ValidatorManager makes sure that msg.sender is allowed
    //       and that claim != bytes32(0)
    /// TODO: add signatures for aggregated claims
    function claim(bytes32 _epochHash) public override {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();

        Result result;
        bytes32[2] memory claims;
        address payable[2] memory claimers;

        Phase currentPhase = Phase(rollupsDS.currentPhase_int);
        uint256 inputAccumulationStart = rollupsDS.inputAccumulationStart;
        uint256 inputDuration = rollupsDS.inputDuration;

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            currentPhase = Phase.AwaitingConsensus;
            rollupsDS.currentPhase_int = uint32(Phase.AwaitingConsensus);
            emit PhaseChange(Phase.AwaitingConsensus);

            // warns input of new epoch
            inputDS.onNewInputAccumulation();
            // update timestamp of sealing epoch proposal
            rollupsDS.sealingEpochTimestamp = uint32(block.timestamp);
        }

        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != AwaitingConsensus"
        );
        (result, claims, claimers) = vmDS.onClaim(
            payable(msg.sender),
            _epochHash
        );

        // emit the claim event before processing it
        // so if the epoch is finalized in this claim (consensus)
        // the number of final epochs doesnt gets contaminated
        emit Claim(
            outputDS.getNumberOfFinalizedEpochs(),
            msg.sender,
            _epochHash
        );

        rollupsDS.resolveValidatorResult(result, claims, claimers);
    }

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() public override {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();

        Phase currentPhase = Phase(rollupsDS.currentPhase_int);
        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != Awaiting Consensus"
        );

        uint256 sealingEpochTimestamp = rollupsDS.sealingEpochTimestamp;
        uint256 challengePeriod = rollupsDS.challengePeriod;
        require(
            block.timestamp > sealingEpochTimestamp + challengePeriod,
            "Challenge period not over"
        );

        require(vmDS.currentClaim != bytes32(0), "No Claim to be finalized");

        rollupsDS.startNewEpoch();
    }

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    //       of finalized epochs array, else there are two non finalized epochs,
    //       one awaiting consensus/dispute and another accumulating input
    function getCurrentEpoch() public view override returns (uint256) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        return rollupsDS.getCurrentEpoch();
    }

    /// @notice returns the current phase
    function getCurrentPhase() public view returns (Phase) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        Phase currentPhase = Phase(rollupsDS.currentPhase_int);
        return currentPhase;
    }

    /// @notice returns the input accumulation start timestamp
    function getInputAccumulationStart() public view returns (uint256) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        uint256 inputAccumulationStart = rollupsDS.inputAccumulationStart;
        return inputAccumulationStart;
    }
}
