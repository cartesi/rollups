// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups Impl
pragma solidity ^0.8.0;

import "./InputImpl.sol";
import "./VoucherImpl.sol";
import "./ValidatorManagerImpl.sol";
import "./Rollups.sol";
import "./DisputeManagerImpl.sol";

contract RollupsImpl is Rollups {
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
    VoucherImpl public voucher; // contract responsible for vouchers
    ValidatorManagerImpl public validatorManager; // contract responsible for validators
    DisputeManagerImpl public disputeManager; // contract responsible for dispute resolution

    struct StorageVar {
        uint32 inputDuration; // duration of input accumulation phase in seconds
        uint32 challengePeriod; // duration of challenge period in seconds
        uint32 inputAccumulationStart; // timestamp when current input accumulation phase started
        uint32 sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable
        uint32 currentPhase_int; // current phase in integer form
    }
    StorageVar storageVar;

    /// @notice functions modified by onlyInputContract can only be called
    // by input contract
    modifier onlyInputContract {
        require(msg.sender == address(input), "only Input Contract");
        _;
    }

    /// @notice functions modified by onlyDisputeContract can only be called
    // by dispute contract
    modifier onlyDisputeContract {
        require(msg.sender == address(disputeManager), "only Dispute Contract");
        _;
    }

    /// @notice creates contract
    /// @param _inputDuration duration of input accumulation phase in seconds
    /// @param _challengePeriod duration of challenge period in seconds
    /// @param _inputLog2Size size of the input drive in this machine
    /// @param _validators initial validator set
    constructor(
        uint256 _inputDuration,
        uint256 _challengePeriod,
        // input constructor variables
        uint256 _inputLog2Size,
        // validator manager constructor variables
        address payable[] memory _validators
    ) {
        input = new InputImpl(address(this), _inputLog2Size);
        voucher = new VoucherImpl(address(this));
        validatorManager = new ValidatorManagerImpl(address(this), _validators);
        disputeManager = new DisputeManagerImpl(address(this));

        storageVar = StorageVar(
            uint32(_inputDuration),
            uint32(_challengePeriod),
            uint32(block.timestamp),
            0,
            uint32(Phase.InputAccumulation)
        );

        emit RollupsCreated(
            address(input),
            address(voucher),
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

        Phase currentPhase = Phase(storageVar.currentPhase_int);
        uint256 inputAccumulationStart = storageVar.inputAccumulationStart;
        uint256 inputDuration = storageVar.inputDuration;

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            currentPhase = Phase.AwaitingConsensus;
            storageVar.currentPhase_int = uint32(Phase.AwaitingConsensus);
            emit PhaseChange(Phase.AwaitingConsensus);

            // warns input of new epoch
            input.onNewInputAccumulation();
            // update timestamp of sealing epoch proposal
            storageVar.sealingEpochTimestamp = uint32(block.timestamp);
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
        emit Claim(voucher.getNumberOfFinalizedEpochs(), msg.sender, _epochHash);

        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() public override {
        Phase currentPhase = Phase(storageVar.currentPhase_int);
        require(
            currentPhase == Phase.AwaitingConsensus,
            "Phase != Awaiting Consensus"
        );

        uint256 sealingEpochTimestamp = storageVar.sealingEpochTimestamp;
        uint256 challengePeriod = storageVar.challengePeriod;
        require(
            block.timestamp > sealingEpochTimestamp + challengePeriod,
            "Challenge period not over"
        );

        require(
            validatorManager.getCurrentClaim() != bytes32(0),
            "No Claim to be finalized"
        );

        startNewEpoch();
    }

    /// @notice called when new input arrives, manages the phase changes
    /// @dev can only be called by input contract
    function notifyInput() public override onlyInputContract returns (bool) {
        Phase currentPhase = Phase(storageVar.currentPhase_int);
        uint256 inputAccumulationStart = storageVar.inputAccumulationStart;
        uint256 inputDuration = storageVar.inputDuration;

        if (
            currentPhase == Phase.InputAccumulation &&
            block.timestamp > inputAccumulationStart + inputDuration
        ) {
            storageVar.currentPhase_int = uint32(Phase.AwaitingConsensus);
            emit PhaseChange(Phase.AwaitingConsensus);
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
        storageVar.sealingEpochTimestamp = uint32(block.timestamp);

        emit ResolveDispute(_winner, _loser, _winningClaim);
        resolveValidatorResult(result, claims, claimers);
    }

    /// @notice starts new epoch
    function startNewEpoch() internal {
        // reset input accumulation start and deactivate challenge period start
        storageVar.currentPhase_int = uint32(Phase.InputAccumulation);
        emit PhaseChange(Phase.InputAccumulation);
        storageVar.inputAccumulationStart = uint32(block.timestamp);
        storageVar.sealingEpochTimestamp = type(uint32).max;

        bytes32 finalClaim = validatorManager.onNewEpoch();

        // emit event before finalized epoch is added to vouchers storage
        emit FinalizeEpoch(voucher.getNumberOfFinalizedEpochs(), finalClaim);

        voucher.onNewEpoch(finalClaim);
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
            Phase currentPhase = Phase(storageVar.currentPhase_int);
            if (currentPhase != Phase.AwaitingConsensus) {
                storageVar.currentPhase_int = uint32(Phase.AwaitingConsensus);
                emit PhaseChange(Phase.AwaitingConsensus);
            }
        } else if (_result == ValidatorManager.Result.Consensus) {
            startNewEpoch();
        } else {
            // for the case when _result == ValidatorManager.Result.Conflict
            Phase currentPhase = Phase(storageVar.currentPhase_int);
            if (currentPhase != Phase.AwaitingDispute) {
                storageVar.currentPhase_int = uint32(Phase.AwaitingDispute);
                emit PhaseChange(Phase.AwaitingDispute);
            }
            disputeManager.initiateDispute(_claims, _claimers);
        }
    }

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    //       of finalized epochs array, else there are two non finalized epochs,
    //       one awaiting consensus/dispute and another accumulating input

    function getCurrentEpoch() public view override returns (uint256) {
        uint256 finalizedEpochs = voucher.getNumberOfFinalizedEpochs();

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

    /// @notice returns address of voucher contract
    function getVoucherAddress() public view override returns (address) {
        return address(voucher);
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
        uint256 inputAccumulationStart = storageVar.inputAccumulationStart;
        return inputAccumulationStart;
    }
}
