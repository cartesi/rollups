// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ClaimsMask library
pragma solidity >=0.8.8;

// ClaimMask is used to keep track of the number of claims (that have been redeemed) for up to 8 validators
// | #claims_validator7 | #claims_validator6 | ... | #claims_validator0 |
// |       32 bits      |       32 bits      | ... |       32 bits      |
type ClaimMask is uint256;

library ClaimsMaskLibrary {
    /// @notice this function returns the #claims that the validator has redeemed
    /// @param  _numClaimsRedeemed the ClaimMask for the number of claims redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    ///     this index can be obtained though `getNumberOfClaimsByIndex` function in Validator Manager
    function getNumClaimsRedeemed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex
    ) public pure returns (uint256) {
        require(_validatorIndex < 8, "index out of range");
        uint256 bitmask = (1 << 32) - 1;
        return
            (ClaimMask.unwrap(_numClaimsRedeemed) >> (32 * _validatorIndex)) &
            bitmask;
    }

    /// @notice this function increases the #claims that the validator has redeemed
    /// @param  _numClaimsRedeemed the ClaimMask for the number of claims redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    /// @param  _value the increase value
    function increaseNumClaimed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex,
        uint256 _value
    ) public pure returns (ClaimMask) {
        require(_validatorIndex < 8, "index out of range");
        uint256 currentNum =
            getNumClaimsRedeemed(_numClaimsRedeemed, _validatorIndex);
        uint256 newNum = currentNum + _value; // by default, solc checks if this line overflows
        return
            setNumClaimsRedeemed(_numClaimsRedeemed, _validatorIndex, newNum);
    }

    /// @notice this function sets the #claims that the validator has redeemed
    /// @param  _numClaimsRedeemed the ClaimMask for the number of claims redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    /// @param  _value the set value
    function setNumClaimsRedeemed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex,
        uint256 _value
    ) public pure returns (ClaimMask) {
        require(_validatorIndex < 8, "index out of range");
        require(_value <= ((1 << 32) - 1), "ClaimMask Overflow");
        uint256 bitmask = ~(((1 << 32) - 1) << (32 * _validatorIndex));
        uint256 clearedClaimMask =
            ClaimMask.unwrap(_numClaimsRedeemed) & bitmask;
        _numClaimsRedeemed = ClaimMask.wrap(
            clearedClaimMask | (_value << (32 * _validatorIndex))
        );
        return _numClaimsRedeemed;
    }

    /// @notice this function creates a new ClaimMask variable an array of 8 values
    /// @param  _value the value array in the order from low -> high (validator# 0->7)
    function newNumClaimsRedeemed(uint256[8] calldata _value)
        public
        pure
        returns (ClaimMask)
    {
        uint256 maskValue;
        for (uint256 i; i < 8; i++) {
            require(_value[i] < (1 << 32), "value out of range");
            maskValue = maskValue | (_value[i] << (32 * i));
        }
        return ClaimMask.wrap(maskValue);
    }
}
