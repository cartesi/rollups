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

import {LibRollupsInit} from "../libraries/LibRollupsInit.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";

contract RollupsInitFacet {
    // @notice initialize the Rollups contract
    // @params _validators initial validator set
    // @dev validators have to be unique, if the same validator is added twice
    //      consensus will never be reached
    function init(
        // Validator Manager parameters
        address payable[] memory _validators
    ) public {
        LibRollupsInit.DiamondStorage storage ds =
            LibRollupsInit.diamondStorage();

        require(!ds.initialized, "Rollups already initialized");
        ds.initialized = true; // avoid reentrancy and reinitialization

        initValidatorManager(_validators);
    }

    // @notice initialize the Validator Manager facet
    // @params _validators initial validator set
    function initValidatorManager(address payable[] memory _validators)
        private
    {
        LibValidatorManager.DiamondStorage storage ds =
            LibValidatorManager.diamondStorage();

        ds.validators = _validators;
        ds.consensusGoalMask = LibValidatorManager.updateConsensusGoalMask();
    }
}
