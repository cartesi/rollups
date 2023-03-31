// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title History
pragma solidity ^0.8.8;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IHistory} from "./IHistory.sol";

contract History is IHistory, Ownable {
    struct Claim {
        bytes32 epochHash;
        uint128 firstIndex;
        uint128 lastIndex;
    }

    // mapping from dapp address => array of claims
    mapping(address => Claim[]) internal claims;

    // Events

    /// @notice A new claim was submitted
    /// @param dapp  The address of the dapp for which the claim was submitted.
    /// @param claim Claim for a specific dapp
    event NewClaimToHistory(address indexed dapp, Claim claim);

    constructor(address _owner) {
        // constructor in Ownable already called `transferOwnership(msg.sender)`, so
        // we only need to call `transferOwnership(_owner)` if _owner != msg.sender
        if (_owner != msg.sender) {
            transferOwnership(_owner);
        }
    }

    function submitClaim(
        bytes calldata _encodedClaim
    ) external override onlyOwner {
        (address dapp, Claim memory claim) = abi.decode(
            _encodedClaim,
            (address, Claim)
        );

        require(claim.firstIndex <= claim.lastIndex, "History: FI > LI");

        Claim[] storage dappClaims = claims[dapp];
        uint256 numDAppClaims = dappClaims.length;

        require(
            numDAppClaims == 0 ||
                (claim.firstIndex > dappClaims[numDAppClaims - 1].lastIndex),
            "History: FI <= previous LI"
        );

        dappClaims.push(claim);

        emit NewClaimToHistory(dapp, claim);
    }

    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        uint256 claimIndex = abi.decode(_proofContext, (uint256));

        Claim memory claim = claims[_dapp][claimIndex];

        return (claim.epochHash, claim.firstIndex, claim.lastIndex);
    }

    // emits an `OwnershipTransfered` event (see `Ownable`)
    function migrateToConsensus(
        address _consensus
    ) external override onlyOwner {
        transferOwnership(_consensus);
    }
}
