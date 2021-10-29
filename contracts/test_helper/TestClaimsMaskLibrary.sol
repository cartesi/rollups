// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Test ClaimsMask library
pragma solidity >=0.8.8;

import "../ClaimsMaskLibrary.sol";

contract TestClaimsMaskLibrary {
    function getNumClaimsRedeemed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex
    ) public pure returns (uint256) {
        return
            ClaimsMaskLibrary.getNumClaimsRedeemed(
                _numClaimsRedeemed,
                _validatorIndex
            );
    }

    function increaseNumClaimed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex,
        uint256 _value
    ) public pure returns (ClaimMask) {
        return
            ClaimsMaskLibrary.increaseNumClaimed(
                _numClaimsRedeemed,
                _validatorIndex,
                _value
            );
    }

    function setNumClaimsRedeemed(
        ClaimMask _numClaimsRedeemed,
        uint256 _validatorIndex,
        uint256 _value
    ) public pure returns (ClaimMask) {
        return
            ClaimsMaskLibrary.setNumClaimsRedeemed(
                _numClaimsRedeemed,
                _validatorIndex,
                _value
            );
    }

    function newNumClaimsRedeemed(uint256 _value)
        public
        pure
        returns (ClaimMask)
    {
        return ClaimsMaskLibrary.newNumClaimsRedeemed(_value);
    }
}
