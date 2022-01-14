// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager facet
pragma solidity ^0.8.0;

import {IValidatorManager} from "../interfaces/IValidatorManager.sol";

import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";

import {ClaimsMaskLibrary, ClaimsMask} from "../ClaimsMaskLibrary.sol";

contract ValidatorManagerFacet is IValidatorManager {
    using LibValidatorManager for LibValidatorManager.DiamondStorage;
    using ClaimsMaskLibrary for ClaimsMask;

    // @notice get agreement mask
    // @return current state of agreement mask
    function getAgreementMask() public view returns (uint256) {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.claimsMask.getAgreementMask();
    }

    // @notice get consensus goal mask
    // @return current consensus goal mask
    function getConsensusGoalMask() public view returns (uint256) {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.claimsMask.getConsensusGoalMask();
    }

    // @notice get current claim
    // @return current claim
    function getCurrentClaim() public view override returns (bytes32) {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.currentClaim;
    }

    // @notice get number of claims the sender has made
    // @params validator address
    // @return #claims
    function getNumberOfClaimsByAddress(address payable _sender)
        public
        view
        returns (uint256)
    {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.getNumberOfClaimsByAddress(_sender);
    }

    // @notice find the validator and return the index or revert
    // @params validator address
    // @return validator index or revert
    function getValidatorIndex(address _sender) public view returns (uint256) {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.getValidatorIndex(_sender);
    }

    // @notice get number of claims by the index in the validator set
    // @params the index in validator set
    // @return #claims
    function getNumberOfClaimsByIndex(uint256 _index)
        public
        view
        returns (uint256)
    {
        LibValidatorManager.DiamondStorage storage vmDS =
            LibValidatorManager.diamondStorage();
        return vmDS.getNumberOfClaimsByIndex(_index);
    }
}
