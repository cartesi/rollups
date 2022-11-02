// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Diamond Initialization Contract
pragma solidity ^0.8.0;

// Rollups-related dependencies
import {Phase} from "../interfaces/IRollups.sol";
import {LibRollups} from "../libraries/LibRollups.sol";
import {LibInput} from "../libraries/LibInput.sol";
import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";
import {LibClaimsMask} from "../libraries/LibClaimsMask.sol";
import {LibFeeManager} from "../libraries/LibFeeManager.sol";
import {IBank} from "../IBank.sol";

// Diamond-related dependencies
import {LibDiamond} from "../libraries/LibDiamond.sol";
import {IDiamondLoupe} from "../interfaces/IDiamondLoupe.sol";
import {IDiamondCut} from "../interfaces/IDiamondCut.sol";
import {IERC173} from "../interfaces/IERC173.sol"; // not in openzeppelin-contracts yet
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

/// @notice diamond configurations
/// @param templateHash state hash of the cartesi machine at t0
/// @param inputDuration duration of input accumulation phase in seconds
/// @param challengePeriod duration of challenge period in seconds
/// @param inputLog2Size size of the input memory range in this machine
/// @param feePerClaim fee per claim to reward the validators
/// @param feeManagerBank fee manager bank address
/// @param feeManagerOwner fee manager owner address
/// @param validators initial validator set
/// @dev validators have to be unique, if the same validator is added twice
///      consensus will never be reached
struct DiamondConfig {
    // RollupsFacet
    bytes32 templateHash;
    uint256 inputDuration;
    uint256 challengePeriod;
    // InputFacet
    uint256 inputLog2Size;
    // FeeManagerFacet
    uint256 feePerClaim;
    address feeManagerBank;
    address feeManagerOwner;
    // ValidatorManagerFacet
    address payable[] validators;
}

contract DiamondInit {
    using LibValidatorManager for LibValidatorManager.DiamondStorage;
    using LibInput for LibInput.DiamondStorage;

    /// @notice initialize the diamond
    /// @param _dConfig diamond configurations
    function init(DiamondConfig calldata _dConfig) external {
        initERC165();
        initValidatorManager(_dConfig.validators);
        initRollups(
            _dConfig.templateHash,
            _dConfig.inputDuration,
            _dConfig.challengePeriod
        );
        initFeeManager(
            _dConfig.feePerClaim,
            _dConfig.feeManagerBank,
            _dConfig.feeManagerOwner
        );
        initInput(_dConfig.inputLog2Size);
    }

    /// @notice initialize ERC165 data
    function initERC165() private {
        LibDiamond.DiamondStorage storage ds = LibDiamond.diamondStorage();
        ds.supportedInterfaces[type(IERC165).interfaceId] = true;
        ds.supportedInterfaces[type(IDiamondCut).interfaceId] = true;
        ds.supportedInterfaces[type(IDiamondLoupe).interfaceId] = true;
        ds.supportedInterfaces[type(IERC173).interfaceId] = true;
    }

    /// @notice initalize the Input facet
    /// @param _inputLog2Size size of the input memory range in this machine
    function initInput(uint256 _inputLog2Size) private {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();

        require(
            _inputLog2Size >= 3 && _inputLog2Size <= 64,
            "Log of input size: [3,64]"
        );

        inputDS.inputDriveSize = (1 << _inputLog2Size);

        // input box gets initialized with one empty input
        // so that the L2 DApp knows it's own address
        inputDS.addInternalInput("");
    }

    /// @notice initialize the Validator Manager facet
    /// @param _validators initial validator set
    function initValidatorManager(
        address payable[] memory _validators
    ) private {
        LibValidatorManager.DiamondStorage
            storage validatorManagerDS = LibValidatorManager.diamondStorage();

        uint256 maxNumValidators = _validators.length;

        require(maxNumValidators <= 8, "up to 8 validators");

        validatorManagerDS.validators = _validators;
        validatorManagerDS.maxNumValidators = maxNumValidators;

        // create a new ClaimsMask, with only the consensus goal set,
        //      according to the number of validators
        validatorManagerDS.claimsMask = LibClaimsMask
            .newClaimsMaskWithConsensusGoalSet(maxNumValidators);
    }

    /// @notice rollups contract initialized
    /// @param inputDuration duration of input accumulation phase in seconds
    /// @param challengePeriod duration of challenge period in seconds
    event RollupsInitialized(uint256 inputDuration, uint256 challengePeriod);

    /// @notice initialize the Rollups facet
    /// @param _templateHash state hash of the cartesi machine at t0
    /// @param _inputDuration duration of input accumulation phase in seconds
    /// @param _challengePeriod duration of challenge period in seconds
    function initRollups(
        bytes32 _templateHash,
        uint256 _inputDuration,
        uint256 _challengePeriod
    ) private {
        LibRollups.DiamondStorage storage rollupsDS = LibRollups
            .diamondStorage();

        rollupsDS.templateHash = _templateHash;
        rollupsDS.inputDuration = uint32(_inputDuration);
        rollupsDS.challengePeriod = uint32(_challengePeriod);
        rollupsDS.inputAccumulationStart = uint32(block.timestamp);
        rollupsDS.currentPhase_int = uint32(Phase.InputAccumulation);

        emit RollupsInitialized(_inputDuration, _challengePeriod);
    }

    /// @notice FeeManagerImpl contract initialized
    /// @param feePerClaim fee per claim to reward the validators
    /// @param feeManagerBank fee manager bank address
    /// @param feeManagerOwner fee manager owner address
    event FeeManagerInitialized(
        uint256 feePerClaim,
        address feeManagerBank,
        address feeManagerOwner
    );

    /// @notice initalize the Fee Manager facet
    /// @param _feePerClaim fee per claim to reward the validators
    /// @param _feeManagerBank fee manager bank address
    /// @param _feeManagerOwner fee manager owner address
    function initFeeManager(
        uint256 _feePerClaim,
        address _feeManagerBank,
        address _feeManagerOwner
    ) private {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();

        feeManagerDS.feePerClaim = _feePerClaim;
        feeManagerDS.bank = IBank(_feeManagerBank);
        feeManagerDS.owner = _feeManagerOwner;

        emit FeeManagerInitialized(
            _feePerClaim,
            _feeManagerBank,
            _feeManagerOwner
        );
    }
}
