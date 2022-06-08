// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Input Box
pragma solidity ^0.8.13;

contract InputBox {
    uint256 immutable inputDriveSize;

    bytes32[] inputBox;

    constructor (uint256 _inputDriveSize) {
        inputDriveSize = _inputDriveSize;
    }

    // calldata!
    event InputAdded(
        uint256 indexed inputIndex,
        address sender,
        uint256 timestamp,
        bytes input
    );
}

// how does the dapp know its own address if input box is shared?
