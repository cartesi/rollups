// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title History
pragma solidity ^0.8.13;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IHistory} from "./IHistory.sol";

contract History is IHistory, Ownable {
    struct Claim {
        bytes32 epochHash;
        uint256 lastClaimInputIndex;
    }

    // mapping from dapp address => FCII => LCII + epoch hash
    mapping(address => mapping(uint256 => Claim)) claims;

    // mapping from dapp address => input index lower bound
    mapping(address => uint256) inputIndexLowerBounds;

    function submitClaim(address _dapp, bytes calldata _claim)
        external
        override
        onlyOwner
    {
        (
            bytes32 epochHash,
            uint256 firstClaimInputIndex,
            uint256 lastClaimInputIndex
        ) = abi.decode(_claim, (bytes32, uint256, uint256));

        require(
            firstClaimInputIndex <= lastClaimInputIndex,
            "History: new FCII > new LCII"
        );

        require(
            firstClaimInputIndex >= inputIndexLowerBounds[_dapp],
            "History: new FCII < IILB"
        );

        inputIndexLowerBounds[_dapp] = lastClaimInputIndex + 1;
        claims[_dapp][firstClaimInputIndex] = Claim({
            epochHash: epochHash,
            lastClaimInputIndex: lastClaimInputIndex
        });

        emit NewClaim(_dapp, _claim);
    }

    function getEpochHash(address _dapp, bytes calldata _claimProof)
        external
        view
        override
        returns (
            bytes32,
            uint256,
            uint256
        )
    {
        (uint256 firstClaimInputIndex, uint256 epochInputIndex) = abi.decode(
            _claimProof,
            (uint256, uint256)
        );

        Claim memory claim = claims[_dapp][firstClaimInputIndex];

        uint256 inputIndex = firstClaimInputIndex + epochInputIndex;

        require(
            inputIndex <= claim.lastClaimInputIndex,
            "History: bad epoch input index"
        );

        return (claim.epochHash, inputIndex, epochInputIndex);
    }

    // emits an `OwnershipTransfered` event (see `Ownable`)
    function migrateToConsensus(address _consensus)
        external
        override
        onlyOwner
    {
        transferOwnership(_consensus);
    }
}
