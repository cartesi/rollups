// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Factory interface
pragma solidity ^0.8.0;

import {CartesiDApp} from "./CartesiDApp.sol";

interface ICartesiDAppFactory {
    /// @notice application configurations
    /// @param diamondOwner diamond owner
    /// @param templateHash state hash of the cartesi machine at t0
    /// @param inputDuration duration of input accumulation phase in seconds
    /// @param challengePeriod duration of challenge period in seconds
    /// @param inputLog2Size size of the input memory range in this machine
    /// @param feePerClaim fee per claim to reward the validators
    /// @param feeManagerOwner fee manager owner address
    /// @param validators initial validator set
    /// @dev validators have to be unique, if the same validator is added twice
    ///      consensus will never be reached
    struct AppConfig {
        // DiamondCutFacet
        address diamondOwner;
        // RollupsFacet
        bytes32 templateHash;
        uint256 inputDuration;
        uint256 challengePeriod;
        // InputFacet
        uint256 inputLog2Size;
        // FeeManagerFacet
        uint256 feePerClaim;
        address feeManagerOwner;
        // ValidatorManagerFacet
        address payable[] validators;
    }

    /// @notice Deploy a new application
    /// @param _appConfig application configurations
    /// @return application address
    function newApplication(
        AppConfig calldata _appConfig
    ) external returns (CartesiDApp);

    /// @notice Event emitted when a new application is deployed
    /// @param application application address
    /// @param config application configurations
    event ApplicationCreated(CartesiDApp indexed application, AppConfig config);
}
