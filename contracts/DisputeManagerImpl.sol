// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title DisputeManager
pragma solidity ^0.8.0;

import "./DisputeManager.sol";
import "./Rollups.sol";

contract DisputeManagerImpl is DisputeManager {
    Rollups immutable rollups; // rollups contract

    /// @notice functions modified by onlyRollups will only be executed if
    //  they're called by Rollups contract, otherwise it will throw an exception
    modifier onlyRollups {
        require(
            msg.sender == address(rollups),
            "Only rollups can call this functions"
        );
        _;
    }

    constructor(address _rollups) {
        rollups = Rollups(_rollups);
    }

    /// @notice initiates a dispute betweent two players
    /// @param _claims conflicting claims
    /// @param _claimers addresses of senders of conflicting claim
    /// @dev this is a mock implementation that just gives the win
    ///      to the address in the first posititon of _claimers array
    function initiateDispute(
        bytes32[2] memory _claims,
        address payable[2] memory _claimers
    ) public override onlyRollups {
        rollups.resolveDispute(_claimers[0], _claimers[1], _claims[0]);
    }
}
