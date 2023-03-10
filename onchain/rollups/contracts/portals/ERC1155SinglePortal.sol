// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Single Transfer Portal
pragma solidity ^0.8.13;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IERC1155SinglePortal} from "./IERC1155SinglePortal.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

contract ERC1155SinglePortal is IERC1155SinglePortal {
    IInputBox internal immutable inputBox;

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function getInputBox() external view override returns (IInputBox) {
        return inputBox;
    }

    function depositSingleERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256 _tokenId,
        uint256 _value,
        bytes calldata _baseLayer,
        bytes calldata _execLayerData
    ) external override {
        _token.safeTransferFrom(
            msg.sender,
            _dapp,
            _tokenId,
            _value,
            _baseLayer
        );

        bytes memory input = InputEncoding.encodeSingleERC1155Deposit(
            _token,
            msg.sender,
            _tokenId,
            _value,
            _baseLayer,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
