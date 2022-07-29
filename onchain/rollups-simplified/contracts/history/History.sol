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

import "@openzeppelin/contracts/access/Ownable.sol";
import {IHistory} from "./IHistory.sol";

contract History is IHistory, Ownable {
    // mapping from dapp address => epoch => claim
    mapping(address => mapping(uint256 => bytes32)) public finalizedClaims;

    constructor(address _consensus) {
        migrateToConsensus(_consensus);
    }

    function submitFinalizedClaim(
        address _dapp,
        uint256 _epoch,
        bytes32 _finalizedClaim,
        uint256 _lastFinalizedInput
    ) external override onlyOwner {
        // overwrite claim even if it's not empty
        finalizedClaims[_dapp][_epoch] = _finalizedClaim;

        emit NewFinalizedClaim(
            _dapp,
            _epoch,
            _finalizedClaim,
            _lastFinalizedInput
        );
    }

    // this is for the case when new consensus uses the same history
    function migrateToConsensus(address _consensus) public override onlyOwner {
        transferOwnership(_consensus);
        emit NewConsensus(_consensus);
    }

    // in this version, the 3rd parameter is ignored
    function getClaim(
        address _dapp,
        uint256 _epoch,
        bytes calldata
    ) external view override returns (bytes32) {
        return finalizedClaims[_dapp][_epoch];
    }
}
