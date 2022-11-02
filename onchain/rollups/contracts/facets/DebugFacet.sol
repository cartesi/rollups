// Copyright 2022 Cartesi Pte. Ltd.

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
import {IEtherPortal} from "../interfaces/IEtherPortal.sol";
import {IERC20Portal} from "../interfaces/IERC20Portal.sol";
import {IERC721Portal} from "../interfaces/IERC721Portal.sol";

import {LibRollups} from "../libraries/LibRollups.sol";
import {LibInput} from "../libraries/LibInput.sol";
import {LibOutput} from "../libraries/LibOutput.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";
import {LibFeeManager} from "../libraries/LibFeeManager.sol";
import {LibClaimsMask, ClaimsMask} from "../libraries/LibClaimsMask.sol";

contract DebugFacet {
    using LibRollups for LibRollups.DiamondStorage;
    using LibInput for LibInput.DiamondStorage;
    using LibOutput for LibOutput.DiamondStorage;
    using LibValidatorManager for LibValidatorManager.DiamondStorage;
    using LibFeeManager for LibFeeManager.DiamondStorage;
    using LibClaimsMask for ClaimsMask;

    function _setCurrentPhase(Phase _phase) public {
        LibRollups.DiamondStorage storage rollupsDS = LibRollups
            .diamondStorage();
        rollupsDS.currentPhase_int = uint32(_phase);
    }

    function _getValidators() public view returns (address payable[] memory) {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();
        return validatorManagerDS.validators;
    }

    function _onClaim(
        address payable _sender,
        bytes32 _claim
    ) public returns (Result, bytes32[2] memory, address payable[2] memory) {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();
        return validatorManagerDS.onClaim(_sender, _claim);
    }

    /// @notice called when a dispute ends in rollups
    /// @param _winner address of dispute winner
    /// @param _loser address of dispute loser
    /// @param _winningClaim the winning claim
    /// @return result of dispute being finished
    function _onDisputeEnd(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    ) public returns (Result, bytes32[2] memory, address payable[2] memory) {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();
        return validatorManagerDS.onDisputeEnd(_winner, _loser, _winningClaim);
    }

    /// @notice called when a new epoch starts
    /// @return current claim
    function _onNewEpochVM() public returns (bytes32) {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();
        return validatorManagerDS.onNewEpoch();
    }

    function _getInputDriveSize() public view returns (uint256) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        return inputDS.inputDriveSize;
    }

    function _etherWithdrawal(bytes calldata _data) public returns (bool) {
        IEtherPortal etherPortal = IEtherPortal(address(this));
        return etherPortal.etherWithdrawal(_data);
    }

    function _onNewEpochOutput(bytes32 epochHash) public {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();
        outputDS.onNewEpoch(epochHash);
    }

    function _erc721Withdrawal(bytes calldata _data) public returns (bool) {
        IERC721Portal erc721Portal = IERC721Portal(address(this));
        return erc721Portal.erc721Withdrawal(_data);
    }

    function _getFeePerClaim() public view returns (uint256) {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        return feeManagerDS.feePerClaim;
    }

    function _setNumClaims(uint256 _validatorIndex, uint256 _value) public {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();
        validatorManagerDS.claimsMask = validatorManagerDS
            .claimsMask
            .setNumClaims(_validatorIndex, _value);
    }

    function _getNumRedeems(
        uint256 _validatorIndex
    ) public view returns (uint256) {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        return feeManagerDS.numClaimsRedeemed.getNumClaims(_validatorIndex);
    }

    /// @notice emitted on Claim received
    event ClaimReceived(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on Dispute end
    event DisputeEnded(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on new Epoch
    event NewEpoch(bytes32 claim);
}
