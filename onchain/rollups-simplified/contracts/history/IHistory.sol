// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title History interface
pragma solidity ^0.8.13;

interface IHistory {
    function submitFinalizedClaim(
        address _dapp,
        uint256 _epoch,
        bytes32 _finalizedClaim,
        uint256 _lastFinalizedInput
    ) external;

    function migrateToConsensus(address _consensus) external;

    function getClaim(
        address _dapp,
        uint256 _epoch,
        bytes calldata
    ) external view returns (bytes32);

    event NewFinalizedClaim(
        address dapp,
        uint256 epoch,
        bytes32 finalizedClaim,
        uint256 lastFinalizedInput
    );

    event NewConsensus(address newConsensus);
}
