// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IERC20Portal} from "./IERC20Portal.sol";
import {InputRelay} from "../inputs/InputRelay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title ERC-20 Portal
///
/// @notice This contract allows anyone to perform transfers of
/// ERC-20 tokens to a DApp while informing the off-chain machine.
contract ERC20Portal is InputRelay, IERC20Portal {
    /// @notice Constructs the portal.
    /// @param _inputBox The input box used by the portal
    constructor(IInputBox _inputBox) InputRelay(_inputBox) {}

    function depositERC20Tokens(
        IERC20 _token,
        address _dapp,
        uint256 _amount,
        bytes calldata _execLayerData
    ) external override {
        bool success = _token.transferFrom(msg.sender, _dapp, _amount);

        bytes memory input = InputEncoding.encodeERC20Deposit(
            success,
            _token,
            msg.sender,
            _amount,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
