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
import {SafeCast} from "@openzeppelin/contracts/utils/math/SafeCast.sol";

import {IHistory} from "./IHistory.sol";

contract History is IHistory, Ownable {
    using SafeCast for uint256;

    struct Claim {
        bytes32 epochHash;
        uint128 firstIndex;
        uint128 lastIndex;
    }

    // mapping from dapp address => array of claims
    mapping(address => Claim[]) claims;

    function submitClaim(address _dapp, bytes calldata _claim)
        external
        override
        onlyOwner
    {
        (bytes32 epochHash, uint256 firstIndex, uint256 lastIndex) = abi.decode(
            _claim,
            (bytes32, uint256, uint256)
        );

        require(firstIndex <= lastIndex, "History: FI > LI");

        Claim[] storage dappClaims = claims[_dapp];
        uint256 numDAppClaims = dappClaims.length;

        if (numDAppClaims > 0) {
            Claim storage prevDAppClaim = dappClaims[numDAppClaims - 1];
            require(
                firstIndex > prevDAppClaim.lastIndex,
                "History: FI <= previous LI"
            );
        }

        dappClaims.push(
            Claim({
                epochHash: epochHash,
                firstIndex: firstIndex.toUint128(),
                lastIndex: lastIndex.toUint128()
            })
        );

        bytes memory eventData = abi.encode(
            numDAppClaims,
            epochHash,
            firstIndex,
            lastIndex
        );
        emit NewClaim(_dapp, eventData);
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
        (uint256 claimIndex, uint256 inputIndex) = abi.decode(
            _claimProof,
            (uint256, uint256)
        );

        Claim memory claim = claims[_dapp][claimIndex];

        require(
            claim.firstIndex <= inputIndex && inputIndex <= claim.lastIndex,
            "History: bad input index"
        );

        uint256 epochInputIndex = inputIndex - claim.firstIndex;

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
