// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Fee Manager facet
pragma solidity >=0.8.8;

import {IBank} from "../IBank.sol";
import {IFeeManager} from "../interfaces/IFeeManager.sol";
import {LibFeeManager} from "../libraries/LibFeeManager.sol";

contract FeeManagerFacet is IFeeManager {
    using LibFeeManager for LibFeeManager.DiamondStorage;

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        require(!feeManagerDS.lock, "reentrancy not allowed");
        feeManagerDS.lock = true;
        _;
        feeManagerDS.lock = false;
    }

    /// @notice this function can be called to check the number of claims that's redeemable for the validator
    /// @param  _validator address of the validator
    function numClaimsRedeemable(
        address _validator
    ) public view override returns (uint256) {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        return feeManagerDS.numClaimsRedeemable(_validator);
    }

    /// @notice this function can be called to check the number of claims that has been redeemed for the validator
    /// @param  _validator address of the validator
    function getNumClaimsRedeemed(
        address _validator
    ) public view override returns (uint256) {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        return feeManagerDS.getNumClaimsRedeemed(_validator);
    }

    /// @notice contract owner can reset the value of fee per claim
    /// @param  _value the new value of fee per claim
    function resetFeePerClaim(uint256 _value) public override {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        feeManagerDS.onlyOwner();
        feeManagerDS.resetFeePerClaim(_value);
    }

    /// @notice this function can be called to redeem fees for validators
    /// @param  _validator address of the validator that is redeeming
    function redeemFee(address _validator) public override noReentrancy {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        feeManagerDS.redeemFee(_validator);
    }

    /// @notice returns the bank used to manage fees
    function getFeeManagerBank() public view override returns (IBank) {
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();
        return feeManagerDS.bank;
    }
}
