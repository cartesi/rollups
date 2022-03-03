// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups interface
pragma solidity >=0.7.0;

// InputAccumulation - Inputs being accumulated for currrent epoch
// AwaitingConsensus - No disagreeing claims (or no claims)
// AwaitingDispute - Waiting for dispute to be over
// inputs received during InputAccumulation will be included in the
// current epoch. Inputs received while WaitingClaims or ChallengesInProgress
// are accumulated for the next epoch
enum Phase {
    InputAccumulation,
    AwaitingConsensus,
    AwaitingDispute
}

interface IRollups {
    /// @notice claim the result of current epoch
    /// @param _epochHash hash of epoch
    /// @dev ValidatorManager makes sure that msg.sender is allowed
    ///      and that claim != bytes32(0)
    /// TODO: add signatures for aggregated claims
    function claim(bytes32 _epochHash) external;

    /// @notice finalize epoch after timeout
    /// @dev can only be called if challenge period is over
    function finalizeEpoch() external;

    /// @notice returns index of current (accumulating) epoch
    /// @return index of current epoch
    /// @dev if phase is input accumulation, then the epoch number is length
    ///      of finalized epochs array, else there are two epochs two non
    ///      finalized epochs, one awaiting consensus/dispute and another
    ///      accumulating input
    function getCurrentEpoch() external view returns (uint256);

    /// @notice claim submitted
    /// @param epochHash claim being submitted by this epoch
    /// @param claimer address of current claimer
    /// @param epochNumber number of the epoch being submitted
    event Claim(
        uint256 indexed epochNumber,
        address claimer,
        bytes32 epochHash
    );

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
}
