// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups library
pragma solidity ^0.8.0;

library LibRollups {
    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("Rollups.diamond.storage");

    struct DiamondStorage {
        uint32 inputDuration; // duration of input accumulation phase in seconds
        uint32 challengePeriod; // duration of challenge period in seconds
        uint32 inputAccumulationStart; // timestamp when current input accumulation phase started
        uint32 sealingEpochTimestamp; // timestamp on when a proposed epoch (claim) becomes challengeable
        uint32 currentPhase_int; // current phase in integer form
    }

    function diamondStorage()
        internal
        pure
        returns (DiamondStorage storage ds)
    {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }
}
