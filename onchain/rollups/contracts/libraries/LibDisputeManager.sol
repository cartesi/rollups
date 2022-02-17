// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Dispute Manager library
pragma solidity ^0.8.0;

import {LibRollups} from "../libraries/LibRollups.sol";

library LibDisputeManager {
    using LibRollups for LibRollups.DiamondStorage;

    /// @notice initiates a dispute betweent two players
    /// @param claims conflicting claims
    /// @param claimers addresses of senders of conflicting claim
    /// @dev this is a mock implementation that just gives the win
    ///      to the address in the first posititon of claimers array
    function initiateDispute(
        bytes32[2] memory claims,
        address payable[2] memory claimers
    ) internal {
        LibRollups.DiamondStorage storage rollupsDS = LibRollups
            .diamondStorage();
        rollupsDS.resolveDispute(claimers[0], claimers[1], claims[0]);
    }
}
