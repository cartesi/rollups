// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups initialization interface
pragma solidity >=0.7.0;

interface IRollupsInit {
    // @notice initialize the Rollups contract
    // @param _inputDuration duration of input accumulation phase in seconds
    // @param _challengePeriod duration of challenge period in seconds
    // @param _inputLog2Size size of the input drive in this machine
    // @param _feePerClaim fee per claim to reward the validators
    // @param _erc20ForFee the ERC-20 used as rewards for validators
    // @param _feeManagerOwner fee manager owner address
    // @param _validators initial validator set
    // @param _erc20Contract specific ERC-20 contract address used by the portal
    // @dev validators have to be unique, if the same validator is added twice
    //      consensus will never be reached
    function init(
        // rollups init variables
        uint256 _inputDuration,
        uint256 _challengePeriod,
        // input init variables
        uint256 _inputLog2Size,
        // fee manager init variables
        uint256 _feePerClaim,
        address _erc20ForFee,
        address _feeManagerOwner,
        // validator manager init variables
        address payable[] memory _validators,
        // specific ERC-20 portal init variables
        address _erc20Contract
    ) external;

    /// @notice rollups contract initialized
    /// @param _inputDuration duration of input accumulation phase in seconds
    /// @param _challengePeriod duration of challenge period in seconds
    event RollupsInitialized(uint256 _inputDuration, uint256 _challengePeriod);

    /// @notice FeeManagerImpl contract initialized
    // @param _feePerClaim fee per claim to reward the validators
    // @param _erc20ForFee the ERC-20 used as rewards for validators
    // @param _feeManagerOwner fee manager owner address
    event FeeManagerInitialized(
        uint256 _feePerClaim,
        address _erc20ForFee,
        address _feeManagerOwner
    );
}
