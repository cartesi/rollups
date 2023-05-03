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

import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

import {IERC721Portal} from "./IERC721Portal.sol";
import {Portal} from "./Portal.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title ERC-721 Portal
///
/// @notice This contract allows anyone to perform transfers of
/// ERC-721 tokens to a DApp while informing the off-chain machine.
contract ERC721Portal is Portal, IERC721Portal {
    /// @notice Constructs the portal.
    /// @param _inputBox The input box used by the portal
    constructor(IInputBox _inputBox) Portal(_inputBox) {}

    function depositERC721Token(
        IERC721 _token,
        address _dapp,
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external override {
        _token.safeTransferFrom(msg.sender, _dapp, _tokenId, _baseLayerData);

        bytes memory input = InputEncoding.encodeERC721Deposit(
            _token,
            msg.sender,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
