// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Fee Manager
pragma solidity >=0.7.0;

interface FeeManager {
    /// @notice this function can only be called by owner to deposit funds as rewards(fees) for validators
    /// @param _ERC20 address of ERC20 token to be deposited
    /// @param _amount amount of tokens to be deposited
    function erc20fund(address _ERC20, uint256 _amount) external;

    /// @notice contract owner can set/reset the value of fee per claim
    /// @param  _value the new value of fee per claim
    function setFeePerClaim(uint256 _value) external;

    /// @notice this function can be called to redeem fees for validators
    /// @param _ERC20 address of ERC20 token
    /// @param  _validator address of the validator that is redeeming
    function claimFee(address _ERC20, address _validator) external;

    // @notice emitted on ERC20 funds deposited
    event ERC20FundDeposited(address _ERC20, uint256 _amount);

    // @notice emitted on ERC20 funds claimed by validator
    event FeeClaimed(address _ERC20, address _validator, uint256 _amount);
}
