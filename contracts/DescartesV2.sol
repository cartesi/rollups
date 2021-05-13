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
pragma solidity >=0.7.0;

import "./Input.sol";
import "./Output.sol";
import "./ValidatorManager.sol";
import "./DescartesV2.sol";

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

    /// @notice get the current phase
    /// @return the current phase
    function getCurrentPhase() external view returns (Phase);

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
    event Claim(uint256 _epochNumber, address _claimer, bytes32 _epochHash);

    /// @notice epoch finalized
    /// @param _epochHash claim being submitted by this epoch
    /// @param _epochNumber number of the epoch being finalized
    event FinalizeEpoch(uint256 _epochNumber, bytes32 _epochHash);

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
