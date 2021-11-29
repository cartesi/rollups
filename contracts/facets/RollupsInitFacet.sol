// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups initialization facet
pragma solidity ^0.8.0;

import {Phase} from "../interfaces/IRollups.sol";
import {IRollupsInit} from "../interfaces/IRollupsInit.sol";

import {LibRollupsInit} from "../libraries/LibRollupsInit.sol";
import {LibRollups} from "../libraries/LibRollups.sol";
import {LibInput} from "../libraries/LibInput.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";

contract RollupsInitFacet is IRollupsInit {
    // @notice initialize the Rollups contract
    // @param _inputDuration duration of input accumulation phase in seconds
    // @param _challengePeriod duration of challenge period in seconds
    // @param _inputLog2Size size of the input drive in this machine
    // @param _validators initial validator set
    // @dev validators have to be unique, if the same validator is added twice
    //      consensus will never be reached
    function init(
        // rollups contructor variables
        uint256 _inputDuration,
        uint256 _challengePeriod,
        // input constructor variables
        uint256 _inputLog2Size,
        // validator manager constructor variables
        address payable[] memory _validators
    ) public override {
        LibRollupsInit.DiamondStorage storage ds =
            LibRollupsInit.diamondStorage();

        require(!ds.initialized, "Rollups already initialized");

        initInput(_inputLog2Size);
        initValidatorManager(_validators);
        initRollups(_inputDuration, _challengePeriod);

        ds.initialized = true;

        emit RollupsInitialized(_inputDuration, _challengePeriod);
    }

    // @notice initalize the Input facet
    // @param _inputLog2Size size of the input drive in this machine
    function initInput(uint256 _inputLog2Size) private {
        LibInput.DiamondStorage storage ds = LibInput.diamondStorage();

        require(
            _inputLog2Size >= 3 && _inputLog2Size <= 64,
            "Log of input size: [3,64]"
        );

        ds.inputDriveSize = (1 << _inputLog2Size);
    }

    // @notice initialize the Validator Manager facet
    // @param _validators initial validator set
    function initValidatorManager(address payable[] memory _validators)
        private
    {
        LibValidatorManager.DiamondStorage storage ds =
            LibValidatorManager.diamondStorage();

        ds.validators = _validators;
        ds.consensusGoalMask = LibValidatorManager.updateConsensusGoalMask();
    }

    // @notice initialize the Rollups facet
    // @param _inputDuration duration of input accumulation phase in seconds
    // @param _challengePeriod duration of challenge period in seconds
    function initRollups(uint256 _inputDuration, uint256 _challengePeriod)
        private
    {
        LibRollups.DiamondStorage storage ds = LibRollups.diamondStorage();

        // Is this optimal?
        ds.inputDuration = uint32(_inputDuration);
        ds.challengePeriod = uint32(_challengePeriod);
        ds.inputAccumulationStart = uint32(block.timestamp);
        ds.currentPhase_int = uint32(Phase.InputAccumulation);
    }
}
