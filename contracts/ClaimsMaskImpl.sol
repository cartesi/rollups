// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ClaimsMaskImpl
pragma solidity >=0.8.8;

// ClaimMask is used to keep track of the number of claims (that have been redeemed) for up to 8 validators
// | #claims_validator7 | #claims_validator6 | ... | #claims_validator0 |
// |       32 bits      |       32 bits      | ... |       32 bits      |
type ClaimMask is uint256;

contract ClaimsMaskImpl{

    // this uint256 is to keep track of the number of claims that have been redeemed for validators
    // | #redeemed_validator7 | #redeemed_validator6 | ... | #redeemed_validator0 |
    // |        32 bits       |        32 bits       | ... |        32 bits       |
    ClaimMask internal numClaimsRedeemed;

    /// @notice this function returns the #claims that the validator has redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    ///     this index can be obtained though `getNumberOfClaimsByIndex` function in Validator Manager
    function getNumClaimsRedeemed(uint256 _validatorIndex) public view returns(uint256){
        uint256 bitmask = (((1<<32)-1) << (32*_validatorIndex));
        return ClaimMask.unwrap(numClaimsRedeemed) & bitmask;
    }

    /// @notice this function increases the #claims that the validator has redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    /// @param  _value the increase value
    function increaseNumClaimed(uint256 _validatorIndex, uint256 _value) internal{
        uint256 bitmask = (((1<<32)-1) << (32*_validatorIndex));
        uint256 currentNum = ClaimMask.unwrap(numClaimsRedeemed) & bitmask;
        uint256 newNum = currentNum + _value;
        setNumClaimsRedeemed(_validatorIndex, newNum);
    }

    /// @notice this function sets the #claims that the validator has redeemed
    /// @param  _validatorIndex index of the validator in the validator array
    /// @param  _value the set value
    function setNumClaimsRedeemed(uint256 _validatorIndex, uint256 _value) internal{
        uint256 bitmask = ~(((1<<32)-1) << (32*_validatorIndex));
        uint256 clearedClaimMask = ClaimMask.unwrap(numClaimsRedeemed) & bitmask;
        numClaimsRedeemed = ClaimMask.wrap(clearedClaimMask | _value);
    }

    /// @notice this function resets the #claims for ALL validators
    /// @param  _value the set value
    function resetNumClaimsRedeemed(uint256 _value) internal{
        numClaimsRedeemed=ClaimMask.wrap(_value);
    }
}
