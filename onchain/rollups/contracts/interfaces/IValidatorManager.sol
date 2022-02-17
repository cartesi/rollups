// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager interface
pragma solidity >=0.7.0;

// NoConflict - No conflicting claims or consensus
// Consensus - All validators had equal claims
// Conflict - Claim is conflicting with previous one
enum Result {
    NoConflict,
    Consensus,
    Conflict
}

// TODO: What is the incentive for validators to not just copy the first claim that arrived?
interface IValidatorManager {
    /// @notice get current claim
    function getCurrentClaim() external view returns (bytes32);

    /// @notice emitted on Claim received
    event ClaimReceived(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on Dispute end
    event DisputeEnded(
        Result result,
        bytes32[2] claims,
        address payable[2] validators
    );

    /// @notice emitted on new Epoch
    event NewEpoch(bytes32 claim);
}
