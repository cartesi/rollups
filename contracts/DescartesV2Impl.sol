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

/// @title Interface DescartesV2 contract
pragma solidity ^0.8.0;

import "./InputImpl.sol";
import "./OutputImpl.sol";
import "./ValidatorManagerImpl.sol";
import "./DescartesV2.sol";
import "./DisputeManagerImpl.sol";

contract DescartesV2Impl is DescartesV2 {
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

    uint256 public immutable inputDuration; // duration of input accumulation phase in seconds
    uint256 public immutable challengePeriod; // duration of challenge period in seconds

    InputImpl public input; // contract responsible for inputs
    OutputImpl public output; // contract responsible for ouputs
    ValidatorManagerImpl public validatorManager; // contract responsible for validators
    DisputeManagerImpl public disputeManager; // contract responsible for dispute resolution

    uint256 public inputAccumulationStart; // timestamp when current input accumulation phase started
    uint256 public sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable

    Phase public currentPhase; // current state

    /// @notice functions modified by onlyInputContract can only be called
    // by input contract
    modifier onlyInputContract {
        require(msg.sender == address(input), "msg.sender != input contract");
        _;
    }

    /// @notice functions modified by onlyDisputeContract can only be called
    // by dispute contract
    modifier onlyDisputeContract {
        require(
            msg.sender == address(disputeManager),
            "msg.sender != dispute manager contract"
        );
        _;
    }

    /// @notice creates contract
    /// @param _inputDuration duration of input accumulation phase in seconds
    /// @param _challengePeriod duration of challenge period in seconds
    /// @param _inputLog2Size size of the input drive in this machine
    /// @param _log2OutputMetadataArrayDriveSize size of the output metadata array
    //                                           drive in this machine
    /// @param _validators initial validator set
    constructor(
        uint256 _inputDuration,
        uint256 _challengePeriod,
        // input constructor variables
        uint8 _inputLog2Size,
        // output constructor variables
        uint8 _log2OutputMetadataArrayDriveSize,
        // validator manager constructor variables
        address payable[] memory _validators
    ) {
        input = new InputImpl(address(this), _inputLog2Size);
        output = new OutputImpl(address(this), _log2OutputMetadataArrayDriveSize);
        validatorManager = new ValidatorManagerImpl(address(this), _validators);
        disputeManager = new DisputeManagerImpl(address(this));

        inputDuration = _inputDuration;
        challengePeriod = _challengePeriod;

        inputAccumulationStart = block.timestamp;
        currentPhase = Phase.InputAccumulation;

        emit DescartesV2Created(
            address(input),
            address(output),
            address(validatorManager),
            address(disputeManager),
            _inputDuration,
            _challengePeriod
        );
    }

    /// @notice claim the result of current epoch
    /// @param _epochHash hash of epoch
    /// @dev ValidatorManager makes sure that msg.sender is allowed
    //       and that claim != bytes32(0)
    /// TODO: add signatures for aggregated claims
    function claim(bytes32 _epochHash) public override {
        ValidatorManager.Result result;
        bytes32[2] memory claims;
        address payable[2] memory claimers;

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            currentPhase = updatePhase(Phase.AwaitingConsensus);

            // warns input of new epoch
            input.onNewInputAccumulation();
            sealingEpochTimestamp = block.timestamp; // update timestamp of sealing epoch proposal
        }
        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != AwaitingConsensus"
        );
        (result, claims, claimers) = validatorManager.onClaim(
            payable(msg.sender),
            _epochHash
        );

        // emit the claim event before processing it
        // so if the epoch is finalized in this claim (consensus)
        // the number of final epochs doesnt gets contaminated
        emit Claim(
            output.getNumberOfFinalizedEpochs(),
            msg.sender,
            _epochHash
        );

        resolveValidatorResult(result, claims, claimers);

    }

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() public override {
        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != Awaiting Consensus"
        );
        require(
            block.timestamp > sealingEpochTimestamp + challengePeriod,
            "Challenge period is not over"
        );
        require(
            validatorManager.getCurrentClaim() != bytes32(0),
            "No Claim to be finalized"
        );
        currentPhase = updatePhase(Phase.InputAccumulation);

        startNewEpoch();
    }

    /// @notice called when new input arrives, manages the phase changes
    /// @dev can only be called by input contract
    function notifyInput() public override onlyInputContract returns (bool) {
        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            currentPhase = updatePhase(Phase.AwaitingConsensus);
            return true;
        }
        return false;
    }

    /// @notice called when a dispute is resolved by the dispute manager
    /// @param _winner winner of dispute
    /// @param _loser loser of dispute
    /// @param _winningClaim initial claim of winning validator
    /// @dev can only be called by the dispute contract
    function resolveDispute(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    ) public override onlyDisputeContract {
        ValidatorManager.Result result;
        bytes32[2] memory claims;
        address payable[2] memory claimers;

        (result, claims, claimers) = validatorManager.onDisputeEnd(
            _winner,
            _loser,
            _winningClaim
        );

        // restart challenge period
        sealingEpochTimestamp = block.timestamp;

        emit ResolveDispute(_winner, _loser, _winningClaim);
        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice starts new epoch
    function startNewEpoch() internal {
        // reset input accumulation start and deactivate challenge period start
        inputAccumulationStart = block.timestamp;
        sealingEpochTimestamp = type(uint256).max;

        bytes32 finalClaim = validatorManager.onNewEpoch();

        // emit event before finalized epoch is added to outputs storage
        emit FinalizeEpoch(output.getNumberOfFinalizedEpochs(), finalClaim);

        output.onNewEpoch(finalClaim);
        input.onNewEpoch();
    }

    /// @notice resolve results returned by validator manager
    /// @param _result result from claim or dispute operation
    /// @param _claims array of claims in case of new conflict
    /// @param _claimers array of claimers in case of new conflict
    function resolveValidatorResult(
        ValidatorManager.Result _result,
        bytes32[2] memory _claims,
        address payable[2] memory _claimers
    ) internal {
        if (_result == ValidatorManager.Result.NoConflict) {
            currentPhase = updatePhase(Phase.AwaitingConsensus);
        } else if (_result == ValidatorManager.Result.Consensus) {
            currentPhase = updatePhase(Phase.InputAccumulation);
            startNewEpoch();
        } else {
            // for the case when _result == ValidatorManager.Result.Conflict
            currentPhase = updatePhase(Phase.AwaitingDispute);
            disputeManager.initiateDispute(_claims, _claimers);
        }
    }

    /// @notice returns phase and emits events
    /// @param _newPhase phase to be returned and emitted
    function updatePhase(Phase _newPhase) internal returns (Phase) {
        if (_newPhase != currentPhase) {
            emit PhaseChange(_newPhase);
        }
        return _newPhase;
    }

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    //       of finalized epochs array, else there are two non finalized epochs,
    //       one awaiting consensus/dispute and another accumulating input

    function getCurrentEpoch() public view override returns (uint256) {
        uint256 finalizedEpochs = output.getNumberOfFinalizedEpochs();

        return currentPhase == Phase.InputAccumulation? finalizedEpochs : finalizedEpochs + 1;
    }

    /// @notice returns address of input contract
    function getInputAddress() public view override returns (address) {
        return address(input);
    }
    /// @notice returns address of output contract
    function getOutputAddress() public view override returns (address) {
        return address(output);
    }

    /// @notice returns address of validator manager contract
    function getValidatorManagerAddress() public view override returns (address) {
        return address(validatorManager);
    }

    /// @notice returns address of dispute manager contract
    function getDisputeManagerAddress() public view override returns (address) {
        return address(disputeManager);
    }
}
