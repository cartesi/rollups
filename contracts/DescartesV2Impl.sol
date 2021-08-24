// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title DescartesV2 Impl
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

    InputImpl public input; // contract responsible for inputs
    OutputImpl public output; // contract responsible for ouputs
    ValidatorManagerImpl public validatorManager; // contract responsible for validators
    DisputeManagerImpl public disputeManager; // contract responsible for dispute resolution

    struct StorageVar {
        uint56 inputDuration; // duration of input accumulation phase in seconds
        uint56 challengePeriod; // duration of challenge period in seconds
        uint56 inputAccumulationStart; // timestamp when current input accumulation phase started
        uint56 sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable
        uint32 currentPhase_int; // current phase in integer form
    }
    StorageVar storageVar;

    enum Storage_index {
        inputDuration,
        challengePeriod,
        inputAccumulationStart,
        sealingEpochTimestamp,
        currentPhase_int
    }

    /// @notice this function reads a property from memory struct
    function readStruct(StorageVar memory storageVar_mem, Storage_index index)
        internal
        pure
        returns (uint256)
    {
        if (index == Storage_index.inputDuration) {
            return uint256(storageVar_mem.inputDuration);
        } else if (index == Storage_index.challengePeriod) {
            return uint256(storageVar_mem.challengePeriod);
        } else if (index == Storage_index.inputAccumulationStart) {
            return uint256(storageVar_mem.inputAccumulationStart);
        } else if (index == Storage_index.sealingEpochTimestamp) {
            return uint256(storageVar_mem.sealingEpochTimestamp);
        } else {
            // Phase in integer form
            return uint256(storageVar_mem.currentPhase_int);
        }
    }

    // @notice this function updates struct in memory
    // this function does NOT write back to storage automatically,
    // because there can be multiple updates before writing back to storage
    function updateStruct(
        StorageVar memory storageVar_mem,
        Storage_index index,
        uint256 value
    ) internal pure returns (StorageVar memory) {
        if (index == Storage_index.inputDuration) {
            storageVar_mem.inputDuration = uint56(value);
            return storageVar_mem;
        } else if (index == Storage_index.challengePeriod) {
            storageVar_mem.challengePeriod = uint56(value);
            return storageVar_mem;
        } else if (index == Storage_index.inputAccumulationStart) {
            storageVar_mem.inputAccumulationStart = uint56(value);
            return storageVar_mem;
        } else if (index == Storage_index.sealingEpochTimestamp) {
            storageVar_mem.sealingEpochTimestamp = uint56(value);
            return storageVar_mem;
        } else {
            // Phase in integer form
            storageVar_mem.currentPhase_int = uint32(value);
            return storageVar_mem;
        }
    }

    // @notice this function writes struct from memory back to storage
    function writeStruct(StorageVar memory storageVar_mem) internal {
        storageVar = storageVar_mem;
    }

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
        output = new OutputImpl(
            address(this),
            _log2OutputMetadataArrayDriveSize
        );
        validatorManager = new ValidatorManagerImpl(address(this), _validators);
        disputeManager = new DisputeManagerImpl(address(this));

        StorageVar memory storageVar_mem;
        updateStruct(
            storageVar_mem,
            Storage_index.inputDuration,
            _inputDuration
        );
        updateStruct(
            storageVar_mem,
            Storage_index.challengePeriod,
            _challengePeriod
        );
        updateStruct(
            storageVar_mem,
            Storage_index.inputAccumulationStart,
            block.timestamp
        );
        updateStruct(
            storageVar_mem,
            Storage_index.currentPhase_int,
            uint256(Phase.InputAccumulation)
        );
        writeStruct(storageVar_mem);

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

        StorageVar memory storageVar_mem = storageVar;
        Phase currentPhase =
            Phase(readStruct(storageVar_mem, Storage_index.currentPhase_int));

        if (currentPhase == Phase.InputAccumulation) {
            uint256 inputAccumulationStart =
                readStruct(
                    storageVar_mem,
                    Storage_index.inputAccumulationStart
                );
            uint256 inputDuration =
                readStruct(storageVar_mem, Storage_index.inputDuration);

            if (block.timestamp > inputAccumulationStart + inputDuration) {
                currentPhase = Phase.AwaitingConsensus;
                updateStruct(
                    storageVar_mem,
                    Storage_index.currentPhase_int,
                    uint256(updatePhase(Phase.AwaitingConsensus))
                );

                // warns input of new epoch
                input.onNewInputAccumulation();
                // update timestamp of sealing epoch proposal
                updateStruct(
                    storageVar_mem,
                    Storage_index.sealingEpochTimestamp,
                    block.timestamp
                );
                writeStruct(storageVar_mem);
            }
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
        emit Claim(output.getNumberOfFinalizedEpochs(), msg.sender, _epochHash);

        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() public override {
        StorageVar memory storageVar_mem = storageVar;

        Phase currentPhase =
            Phase(readStruct(storageVar_mem, Storage_index.currentPhase_int));
        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != Awaiting Consensus"
        );

        uint256 sealingEpochTimestamp =
            uint256(
                readStruct(storageVar_mem, Storage_index.sealingEpochTimestamp)
            );
        uint256 challengePeriod =
            uint256(readStruct(storageVar_mem, Storage_index.challengePeriod));
        require(
            block.timestamp > sealingEpochTimestamp + challengePeriod,
            "Challenge period is not over"
        );

        require(
            validatorManager.getCurrentClaim() != bytes32(0),
            "No Claim to be finalized"
        );

        updateStruct(
            storageVar_mem,
            Storage_index.currentPhase_int,
            uint256(updatePhase(Phase.InputAccumulation))
        );
        writeStruct(storageVar_mem);

        startNewEpoch();
    }

    /// @notice called when new input arrives, manages the phase changes
    /// @dev can only be called by input contract
    function notifyInput() public override onlyInputContract returns (bool) {
        StorageVar memory storageVar_mem = storageVar;
        Phase currentPhase =
            Phase(readStruct(storageVar_mem, Storage_index.currentPhase_int));
        uint256 inputAccumulationStart =
            uint256(
                readStruct(storageVar_mem, Storage_index.inputAccumulationStart)
            );
        uint256 inputDuration =
            uint256(readStruct(storageVar_mem, Storage_index.inputDuration));

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            updateStruct(
                storageVar_mem,
                Storage_index.currentPhase_int,
                uint256(updatePhase(Phase.AwaitingConsensus))
            );
            writeStruct(storageVar_mem);
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
        storageVar.sealingEpochTimestamp = uint56(block.timestamp);

        emit ResolveDispute(_winner, _loser, _winningClaim);
        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice starts new epoch
    function startNewEpoch() internal {
        // reset input accumulation start and deactivate challenge period start
        StorageVar memory storageVar_mem = storageVar;
        updateStruct(
            storageVar_mem,
            Storage_index.inputAccumulationStart,
            block.timestamp
        );
        updateStruct(
            storageVar_mem,
            Storage_index.sealingEpochTimestamp,
            type(uint256).max
        );
        writeStruct(storageVar_mem);

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
            storageVar.currentPhase_int = uint32(
                updatePhase(Phase.AwaitingConsensus)
            );
        } else if (_result == ValidatorManager.Result.Consensus) {
            storageVar.currentPhase_int = uint32(
                updatePhase(Phase.InputAccumulation)
            );
            startNewEpoch();
        } else {
            // for the case when _result == ValidatorManager.Result.Conflict
            storageVar.currentPhase_int = uint32(
                updatePhase(Phase.AwaitingDispute)
            );
            disputeManager.initiateDispute(_claims, _claimers);
        }
    }

    /// @notice returns phase and emits events
    /// @param _newPhase phase to be returned and emitted
    function updatePhase(Phase _newPhase) internal returns (Phase) {
        if (_newPhase != Phase(storageVar.currentPhase_int)) {
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

        Phase currentPhase = Phase(storageVar.currentPhase_int);

        return
            currentPhase == Phase.InputAccumulation
                ? finalizedEpochs
                : finalizedEpochs + 1;
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
    function getValidatorManagerAddress()
        public
        view
        override
        returns (address)
    {
        return address(validatorManager);
    }

    /// @notice returns address of dispute manager contract
    function getDisputeManagerAddress() public view override returns (address) {
        return address(disputeManager);
    }

    /// @notice returns the current phase
    function getCurrentPhase() public view returns (Phase) {
        Phase currentPhase = Phase(storageVar.currentPhase_int);
        return currentPhase;
    }

    /// @notice returns the input accumulation start timestamp
    function getInputAccumulationStart() public view returns (uint256) {
        uint256 inputAccumulationStart =
            uint256(storageVar.inputAccumulationStart);
        return inputAccumulationStart;
    }
}
