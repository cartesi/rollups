// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input library
pragma solidity ^0.8.0;

library LibInput {
    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("Input.diamond.storage");

    struct DiamondStorage {
        // always needs to keep track of two input boxes:
        // 1 for the input accumulation of next epoch
        // and 1 for the messages during current epoch. To save gas we alternate
        // between inputBox0 and inputBox1
        bytes32[] inputBox0;
        bytes32[] inputBox1;
        uint256 inputDriveSize; // size of input flashdrive
        uint256 currentInputBox;
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
