// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-721 Portal
pragma solidity ^0.8.13;

import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

import {IERC721Portal} from "./IERC721Portal.sol";
import {InputBox} from "../inputs/InputBox.sol";
import {InputHeaders} from "../common/InputHeaders.sol";

contract ERC721Portal is IERC721Portal {
    InputBox public immutable inputBox;

    constructor(InputBox _inputBox) {
        inputBox = _inputBox;
    }

    function depositERC721Token(
        IERC721 _token,
        address _dapp,
        uint256 _tokenId,
        bytes calldata _data
    ) external override {
        // We add the input first to avoid reentrancy attacks
        // as NFT transfers to contracts trigger a special
        // callback function that can call this function again
        bytes memory input = abi.encodePacked(
            InputHeaders.ERC721_DEPOSIT, // Header (1B)
            _token, //                      Token contract (20B)
            msg.sender, //                  Token sender (20B)
            _tokenId, //                    Token identifier (32B)
            _data //                        L2 data (arbitrary size)
        );
        inputBox.addInput(_dapp, input);

        _token.safeTransferFrom(msg.sender, _dapp, _tokenId);
    }
}
