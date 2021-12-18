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
    function newClaimsMask(uint256 _value) public view returns (ClaimsMask) {
        return ClaimsMaskLibrary.newClaimsMask(_value);
    }

    function newClaimsMaskWithConsensusGoalSet(uint256 _numValidators)
        public
        view
        returns (ClaimsMask)
    {
        return
            ClaimsMaskLibrary.newClaimsMaskWithConsensusGoalSet(_numValidators);
    }

    function getNumClaims(ClaimsMask _claimsMask, uint256 _validatorIndex)
        public
        view
        returns (uint256)
    {
        return ClaimsMaskLibrary.getNumClaims(_claimsMask, _validatorIndex);
    }

    function increaseNumClaims(
        ClaimsMask _claimsMask,
        uint256 _validatorIndex,
        uint256 _value
    ) public view returns (ClaimsMask) {
        return
            ClaimsMaskLibrary.increaseNumClaims(
                _claimsMask,
                _validatorIndex,
                _value
            );
    }

    function setNumClaims(
        ClaimsMask _claimsMask,
        uint256 _validatorIndex,
        uint256 _value
    ) public view returns (ClaimsMask) {
        return
            ClaimsMaskLibrary.setNumClaims(
                _claimsMask,
                _validatorIndex,
                _value
            );
    }

    function clearAgreementMask(ClaimsMask _claimsMask)
        public
        view
        returns (ClaimsMask)
    {
        return ClaimsMaskLibrary.clearAgreementMask(_claimsMask);
    }

    function getAgreementMask(ClaimsMask _claimsMask)
        public
        view
        returns (uint256)
    {
        return ClaimsMaskLibrary.getAgreementMask(_claimsMask);
    }

    function setAgreementMask(ClaimsMask _claimsMask, uint256 _validatorIndex)
        public
        view
        returns (ClaimsMask)
    {
        return ClaimsMaskLibrary.setAgreementMask(_claimsMask, _validatorIndex);
    }

    function getConsensusGoalMask(ClaimsMask _claimsMask)
        public
        view
        returns (uint256)
    {
        return ClaimsMaskLibrary.getConsensusGoalMask(_claimsMask);
    }

    function removeValidator(ClaimsMask _claimsMask, uint256 _validatorIndex)
        public
        view
        returns (ClaimsMask)
    {
        return ClaimsMaskLibrary.removeValidator(_claimsMask, _validatorIndex);
    }
}
