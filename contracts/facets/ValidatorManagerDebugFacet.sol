// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager debug facet
pragma solidity ^0.8.0;

import {Result} from "../interfaces/IValidatorManager.sol";

import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";

contract ValidatorManagerDebugFacet {
    using LibValidatorManager for LibValidatorManager.DiamondStorage;

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
}
