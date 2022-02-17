// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Fee Manager library
pragma solidity ^0.8.0;

import {LibValidatorManager} from "../libraries/LibValidatorManager.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {LibClaimsMask, ClaimsMask} from "../libraries/LibClaimsMask.sol";

library LibFeeManager {
    using LibValidatorManager for LibValidatorManager.DiamondStorage;
    using LibFeeManager for LibFeeManager.DiamondStorage;
    using LibClaimsMask for ClaimsMask;

    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("FeeManager.diamond.storage");

    struct DiamondStorage {
        address owner; // owner of Fee Manager
        uint256 feePerClaim;
        IERC20 token; // the token that is used for paying fees to validators
        bool lock; // reentrancy lock
        // A bit set used for up to 8 validators.
        // The first 16 bits are not used to keep compatibility with the validator manager contract.
        // The following every 30 bits are used to indicate the number of total claims each validator has made
        // |     not used    | #claims_validator7 | #claims_validator6 | ... | #claims_validator0 |
        // |     16 bits     |      30 bits       |      30 bits       | ... |      30 bits       |
        ClaimsMask numClaimsRedeemed;
    }

    function diamondStorage()
        internal
        pure
        returns (DiamondStorage storage feeManagerDS)
    {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            feeManagerDS.slot := position
        }
    }

    function onlyOwner(DiamondStorage storage ds) internal view {
        require(ds.owner == msg.sender, "caller is not the owner");
    }

    /// @notice this function can be called to check the number of claims that's redeemable for the validator
    /// @param  feeManagerDS pointer to FeeManager's diamond storage
    /// @param  _validator address of the validator
    function numClaimsRedeemable(
        DiamondStorage storage feeManagerDS,
        address _validator
    ) internal view returns (uint256) {
        require(_validator != address(0), "address should not be 0");

        LibValidatorManager.DiamondStorage
            storage ValidatorManagerDS = LibValidatorManager.diamondStorage();
        uint256 valIndex = ValidatorManagerDS.getValidatorIndex(_validator); // will revert if not found
        uint256 totalClaims = ValidatorManagerDS.claimsMask.getNumClaims(
            valIndex
        );
        uint256 redeemedClaims = feeManagerDS.numClaimsRedeemed.getNumClaims(
            valIndex
        );

        return totalClaims - redeemedClaims; // underflow checked by default with sol0.8
    }

    /// @notice this function can be called to check the number of claims that has been redeemed for the validator
    /// @param  feeManagerDS pointer to FeeManager's diamond storage
    /// @param  _validator address of the validator
    function getNumClaimsRedeemed(
        DiamondStorage storage feeManagerDS,
        address _validator
    ) internal view returns (uint256) {
        require(_validator != address(0), "address should not be 0");

        LibValidatorManager.DiamondStorage
            storage ValidatorManagerDS = LibValidatorManager.diamondStorage();
        uint256 valIndex = ValidatorManagerDS.getValidatorIndex(_validator); // will revert if not found
        uint256 redeemedClaims = feeManagerDS.numClaimsRedeemed.getNumClaims(
            valIndex
        );

        return redeemedClaims;
    }

    /// @notice contract owner can reset the value of fee per claim
    /// @param  feeManagerDS pointer to FeeManager's diamond storage
    /// @param  _value the new value of fee per claim
    function resetFeePerClaim(
        DiamondStorage storage feeManagerDS,
        uint256 _value
    ) internal {
        // before resetting the feePerClaim, pay fees for all validators as per current rates
        LibValidatorManager.DiamondStorage
            storage ValidatorManagerDS = LibValidatorManager.diamondStorage();
        for (uint256 i; i < ValidatorManagerDS.maxNumValidators; i++) {
            address validator = ValidatorManagerDS.validators[i];
            if (
                validator != address(0) &&
                feeManagerDS.numClaimsRedeemable(validator) > 0
            ) {
                feeManagerDS.redeemFee(validator);
            }
        }
        feeManagerDS.feePerClaim = _value;
        emit FeePerClaimReset(_value);
    }

    /// @notice this function can be called to redeem fees for validators
    /// @param  feeManagerDS pointer to FeeManager's diamond storage
    /// @param  _validator address of the validator that is redeeming
    function redeemFee(DiamondStorage storage feeManagerDS, address _validator)
        internal
    {
        // follow the Checks-Effects-Interactions pattern for security

        // ** checks **
        uint256 nowRedeemingClaims = feeManagerDS.numClaimsRedeemable(
            _validator
        );
        require(nowRedeemingClaims > 0, "nothing to redeem yet");

        // ** effects **
        LibValidatorManager.DiamondStorage
            storage ValidatorManagerDS = LibValidatorManager.diamondStorage();
        uint256 valIndex = ValidatorManagerDS.getValidatorIndex(_validator); // will revert if not found
        feeManagerDS.numClaimsRedeemed = feeManagerDS
            .numClaimsRedeemed
            .increaseNumClaims(valIndex, nowRedeemingClaims);

        // ** interactions **
        uint256 feesToSend = nowRedeemingClaims * feeManagerDS.feePerClaim; // number of erc20 tokens to send
        require(
            feeManagerDS.token.transfer(_validator, feesToSend),
            "Failed to redeem fees"
        );
        emit FeeRedeemed(_validator, feesToSend);
    }

    /// @notice emitted on resetting feePerClaim
    event FeePerClaimReset(uint256 _value);

    /// @notice emitted on ERC20 funds redeemed by validator
    event FeeRedeemed(address _validator, uint256 _amount);
}
