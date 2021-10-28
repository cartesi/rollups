// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Fee Manager Impl
pragma solidity >=0.8.8;

import "./FeeManager.sol";
import "./ClaimsMaskLibrary.sol";
import "./ValidatorManagerClaimsCountedImpl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// this FeeManagerImpl manages for up to 8 validators
contract FeeManagerImpl is FeeManager {
    ValidatorManagerClaimsCountedImpl ValidatorManagerCCI;
    ClaimMask numClaimsRedeemed;
    uint256 public feePerClaim;
    IERC20 token; // the token that is used for paying fees to validators
    address owner;
    bool lock; // reentrancy lock

    /// @notice functions modified by onlyowner can only be accessed by contract owner
    modifier onlyOwner {
        require(owner == msg.sender, "only owner");
        _;
    }

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    /// @notice creates FeeManagerImpl contract
    /// @param _ValidatorManagerCCI address of ValidatorManagerClaimsCountedImpl
    /// @param _ERC20 address of erc20 token
    /// @param _feePerClaim set the value of feePerClaim during construction
    constructor(
        address _ValidatorManagerCCI,
        address _ERC20,
        uint256 _feePerClaim
    ) {
        owner = msg.sender;
        ValidatorManagerCCI = ValidatorManagerClaimsCountedImpl(
            _ValidatorManagerCCI
        );
        token = IERC20(_ERC20);
        feePerClaim = _feePerClaim;
        emit FeeManagerCreated(_ValidatorManagerCCI, _ERC20, _feePerClaim);
    }

    /// @notice this function can only be called to deposit funds as rewards(fees) for validators
    /// @param _amount amount of tokens to be deposited
    function erc20fund(uint256 _amount) public override {
        require(
            token.transferFrom(owner, address(this), _amount),
            "erc20 fund deposit failed"
        );
        emit ERC20FundDeposited(_amount);
    }

    /// @notice this function can be called to check the number of claims that's redeemable for the validator
    /// @param  _validator address of the validator
    function numClaimsRedeemable(address _validator)
        public
        view
        override
        returns (uint256)
    {
        require(_validator != address(0), "address should not be 0");
        uint256 valIndex = ValidatorManagerCCI.getValidatorIndex(_validator); // will revert if not found
        uint256 totalClaims =
            ValidatorManagerCCI.getNumberOfClaimsByIndex(valIndex);
        uint256 redeemedClaims =
            ClaimsMaskLibrary.getNumClaimsRedeemed(numClaimsRedeemed, valIndex);

        return totalClaims - redeemedClaims; // underflow checked by default with sol0.8
    }

    /// @notice contract owner can reset the value of fee per claim
    ///         validators should be paid for the past unpaid claims before setting the new fee value
    /// @param  _value the new value of fee per claim
    function resetFeePerClaim(uint256 _value) public override onlyOwner {
        // before resetting the feePerClaim, pay fees for all validators as per current rates
        for (uint256 i; i < ValidatorManagerCCI.maxNumValidators(); i++) {
            address validator = ValidatorManagerCCI.validators(i);
            if (validator != address(0) && numClaimsRedeemable(validator) > 0) {
                claimFee(validator);
            }
        }
        feePerClaim = _value;
        emit FeePerClaimReset(_value);
    }

    /// @notice this function can be called to redeem fees for validators
    /// @param  _validator address of the validator that is redeeming
    function claimFee(address _validator) public override noReentrancy {
        // follow the Checks-Effects-Interactions pattern for security

        // ** checks **
        uint256 nowRedeemingClaims = numClaimsRedeemable(_validator);
        require(nowRedeemingClaims > 0, "nothing to redeem yet");

        // ** effects **
        uint256 valIndex = ValidatorManagerCCI.getValidatorIndex(_validator); // will revert if not found
        numClaimsRedeemed = ClaimsMaskLibrary.increaseNumClaimed(
            numClaimsRedeemed,
            valIndex,
            nowRedeemingClaims
        );

        // ** interactions **
        uint256 feesToSend = nowRedeemingClaims * feePerClaim; // number of erc20 tokens to send
        require(token.transfer(_validator, feesToSend), "Failed to claim fees");

        emit FeeClaimed(_validator, feesToSend);
    }
}
