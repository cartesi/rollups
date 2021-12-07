// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Debug facet
pragma solidity ^0.8.0;

import {Result} from "../interfaces/IValidatorManager.sol";
import {Phase} from "../interfaces/IRollups.sol";

import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";
import {LibRollups} from "../libraries/LibRollups.sol";

contract DebugFacet {
    using LibRollups for LibRollups.DiamondStorage;
    using LibValidatorManager for LibValidatorManager.DiamondStorage;

    function _setInputAccumulationStart(uint32 _inputAccumulationStart) public {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        rollupsDS.inputAccumulationStart = _inputAccumulationStart;
    }

    function _setCurrentPhase(Phase _phase) public {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        rollupsDS.currentPhase_int = uint32(_phase);
    }

    function _getCurrentPhase() public view returns (Phase _phase) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        return Phase(rollupsDS.currentPhase_int);
    }

    function _getCurrentEpoch() public view returns (uint256) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        return rollupsDS.getCurrentEpoch();
    }

    function _getValidators()
        public
        view
        returns (address payable[] memory validators)
    {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.validators;
    }

    function _onClaim(address payable _sender, bytes32 _claim)
        public
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.onClaim(_sender, _claim);
    }

    // @notice called when a dispute ends in rollups
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function _onDisputeEnd(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    )
        public
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.onDisputeEnd(_winner, _loser, _winningClaim);
    }

    // @notice called when a new epoch starts
    // @return current claim
    function _onNewEpoch() public returns (bytes32) {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.onNewEpoch();
    }

    // @notice emitted on Claim received
    event ClaimReceived(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    // @notice emitted on Dispute end
    event DisputeEnded(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    // @notice emitted on new Epoch
    event NewEpoch(bytes32 claim);
}
