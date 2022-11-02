// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Fee Manager interface
pragma solidity >=0.7.0;

import {IBank} from "../IBank.sol";

interface IFeeManager {
    /// @notice this function can be called to check the number of claims that's redeemable for the validator
    /// @param  _validator address of the validator
    function numClaimsRedeemable(
        address _validator
    ) external view returns (uint256);

    /// @notice this function can be called to check the number of claims that has been redeemed for the validator
    /// @param  _validator address of the validator
    function getNumClaimsRedeemed(
        address _validator
    ) external view returns (uint256);

    /// @notice contract owner can set/reset the value of fee per claim
    /// @param  _value the new value of fee per claim
    function resetFeePerClaim(uint256 _value) external;

    /// @notice this function can be called to redeem fees for validators
    /// @param  _validator address of the validator that is redeeming
    function redeemFee(address _validator) external;

    /// @notice returns the bank used to manage fees
    function getFeeManagerBank() external view returns (IBank);

    /// @notice emitted on resetting feePerClaim
    event FeePerClaimReset(uint256 value);

    /// @notice emitted on ERC20 funds redeemed by validator
    event FeeRedeemed(address validator, uint256 claims);
}
