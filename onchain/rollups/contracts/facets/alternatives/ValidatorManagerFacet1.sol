// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager facet (alternative version)
pragma solidity ^0.8.0;

import {IValidatorManager} from "../../interfaces/IValidatorManager.sol";

import {LibValidatorManager1} from "../../libraries/alternatives/LibValidatorManager1.sol";

contract ValidatorManagerFacet1 is IValidatorManager {
    /// @notice get agreement mask
    /// @return current state of agreement mask
    function getCurrentAgreementMask() public view returns (uint32) {
        LibValidatorManager1.DiamondStorage
            storage validatorManagerDS = LibValidatorManager1.diamondStorage();
        return validatorManagerDS.claimAgreementMask;
    }

    /// @notice get consensus goal mask
    /// @return current consensus goal mask
    function getConsensusGoalMask() public view returns (uint32) {
        LibValidatorManager1.DiamondStorage
            storage validatorManagerDS = LibValidatorManager1.diamondStorage();
        return validatorManagerDS.consensusGoalMask;
    }

    /// @notice get current claim
    /// @return current claim
    function getCurrentClaim() public view override returns (bytes32) {
        LibValidatorManager1.DiamondStorage
            storage validatorManagerDS = LibValidatorManager1.diamondStorage();
        return validatorManagerDS.currentClaim;
    }
}
