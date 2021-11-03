// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Interface DescartesV2 contract
pragma solidity >=0.7.0;

import "./Input.sol";
import "./Output.sol";
import "./ValidatorManager.sol";

interface DescartesV2 {
    // InputAccumulation - Inputs being accumulated for currrent epoch
    // AwaitingConsensus - No disagreeing claims (or no claims)
    // AwaitingDispute - Waiting for dispute to be over
    // inputs received during InputAccumulation will be included in the
    // current epoch. Inputs received while WaitingClaims or ChallengesInProgress
    // are accumulated for the next epoch
    enum Phase {InputAccumulation, AwaitingConsensus, AwaitingDispute}

    /// @notice claim the result of current epoch
    /// @param _epochHash hash of epoch
    /// @dev ValidatorManager makes sure that msg.sender is allowed
    //       and that claim != bytes32(0)
    /// TODO: add signatures for aggregated claims
    function claim(bytes32 _epochHash) external;

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() external;

    /// @notice called when new input arrives, manages the phase changes
    /// @dev can only be called by input contract
    function notifyInput() external returns (bool);

    /// @notice called when a dispute is resolved by the dispute manager
    /// @param _winner winner of dispute
    /// @param _loser lose of sipute
    /// @param _winningClaim initial claim of winning validator
    /// @dev can only by the dispute contract
    function resolveDispute(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    ) external;

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    //       of finalized epochs array, else there are two epochs two non
    //       finalized epochs, one awaiting consensus/dispute and another
    //      accumulating input
    function getCurrentEpoch() external view returns (uint256);

    /// @notice returns address of input contract
    function getInputAddress() external view returns (address);

    /// @notice returns address of output contract
    function getOutputAddress() external view returns (address);

    /// @notice returns address of validator manager contract
    function getValidatorManagerAddress() external view returns (address);

    /// @notice returns address of dispute manager contract
    function getDisputeManagerAddress() external view returns (address);

    // events

    /// @notice contract created
    /// @param _input address of input contract
    /// @param _output address of output contract
    /// @param _validatorManager address of validatorManager contract
    /// @param _disputeManager address of disputeManager contract
    /// @param _inputDuration duration of input accumulation phase in seconds
    /// @param _challengePeriod duration of challenge period in seconds
    event DescartesV2Created(
        address _input,
        address _output,
        address _validatorManager,
        address _disputeManager,
        uint256 _inputDuration,
        uint256 _challengePeriod
    );

    /// @notice claim submitted
    /// @param _epochHash claim being submitted by this epoch
    /// @param _claimer address of current claimer
    /// @param _epochNumber number of the epoch being submitted
    event Claim(uint256 indexed _epochNumber, address _claimer, bytes32 _epochHash);

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
}
