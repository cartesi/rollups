// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Box
pragma solidity ^0.8.13;

import {IInputBox} from "./IInputBox.sol";
import {LibInput} from "../library/LibInput.sol";

contract InputBox is IInputBox {
    mapping(address => bytes32[]) inputBoxes;

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
