// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Portal
pragma solidity ^0.8.13;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IERC1155Portal} from "./IERC1155Portal.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

contract ERC1155Portal is IERC1155Portal {
    IInputBox immutable inputBox;

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

    function depositBatchERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256[] calldata _tokenIds,
        uint256[] calldata _values,
        bytes calldata _baseLayer,
        bytes calldata _execLayerData
    ) external override {
        _token.safeBatchTransferFrom(
            msg.sender,
            _dapp,
            _tokenIds,
            _values,
            _baseLayer
        );

        bytes memory input = InputEncoding.encodeBatchERC1155Deposit(
            _token,
            msg.sender,
            _tokenIds,
            _values,
            _baseLayer,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
