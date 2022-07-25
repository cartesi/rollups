// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-20 Portal
pragma solidity ^0.8.13;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IERC20Portal} from "./IERC20Portal.sol";
import {InputBox} from "../inputs/InputBox.sol";
import {InputHeaders} from "../common/InputHeaders.sol";

contract ERC20Portal is IERC20Portal {
    InputBox public immutable inputBox;

    constructor(address _inputBox) {
        inputBox = InputBox(_inputBox);
    }

    function depositERC20Tokens(
        address _token,
        address _dapp,
        uint256 _amount,
        bytes calldata _data
    ) external override {
        bool success = IERC20(_token).transferFrom(msg.sender, _dapp, _amount);
        bytes1 header = success
            ? InputHeaders.ERC20_DEPOSIT_TRUE
            : InputHeaders.ERC20_DEPOSIT_FALSE;
        bytes memory input = abi.encodePacked(
            header, //     Header (1B)
            _dapp, //      DApp contract (20B)
            _token, //     Token contract (20B)
            msg.sender, // Token sender (20B)
            _amount, //    Amount of tokens (32B)
            _data //       L2 data (arbitrary size)
        );
        inputBox.addIndirectInput(input);
    }
}
