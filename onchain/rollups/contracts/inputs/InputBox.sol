// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {IInputBox} from "./IInputBox.sol";
import {LibInput} from "../library/LibInput.sol";

/// @title Input Box
///
/// @notice Trustless and permissionless contract that receives arbitrary blobs
/// (called "inputs") from anyone and adds a compound hash to an append-only list
/// (called "input box"). Each DApp has its own input box.
///
/// The hash that is stored on-chain is composed by the hash of the input blob,
/// the block number and timestamp, the input sender address, and the input index.
///
/// Data availability is guaranteed by the emission of `InputAdded` events
/// on every successful call to `addInput`. This ensures that inputs can be
/// retrieved by anyone at any time, without having to rely on centralized data
/// providers.
///
/// From the perspective of this contract, inputs are encoding-agnostic byte
/// arrays. It is up to the DApp to interpret, validate and act upon inputs.
contract InputBox is IInputBox {
    /// @notice Mapping from DApp address to list of input hashes.
    /// @dev See the `getNumberOfInputs`, `getInputHash` and `addInput` functions.
    mapping(address => bytes32[]) internal inputBoxes;

    function addInput(
        address _dapp,
        bytes calldata _input
    ) external override returns (bytes32) {
        bytes32[] storage inputBox = inputBoxes[_dapp];
        uint256 inboxInputIndex = inputBox.length;

        bytes32 inputHash = LibInput.computeInputHash(
            msg.sender,
            block.number,
            block.timestamp,
            _input,
            inboxInputIndex
        );

        // add input to correct inbox
        inputBox.push(inputHash);

        // block.number and timestamp can be retrieved by the event metadata itself
        emit InputAdded(_dapp, inboxInputIndex, msg.sender, _input);

        return inputHash;
    }

    function getNumberOfInputs(
        address _dapp
    ) external view override returns (uint256) {
        return inputBoxes[_dapp].length;
    }

    function getInputHash(
        address _dapp,
        uint256 _index
    ) external view override returns (bytes32) {
        return inputBoxes[_dapp][_index];
    }
}
