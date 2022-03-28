// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Bank interface
pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IBank {
    /// @notice returns the token used internally
    function getToken() external view returns (IERC20);

    /// @notice get balance of `_owner`
    /// @param _owner account owner
    function balanceOf(address _owner) external view returns (uint256);

    /// @notice transfer `_value` tokens from bank to `_to`
    /// @notice decrease the balance of caller by `_value`
    /// @param _to account that will receive `_value` tokens
    /// @param _value amount of tokens to be transfered
    function transferTokens(address _to, uint256 _value) external;

    /// @notice transfer `_value` tokens from caller to bank
    /// @notice increase the balance of `_to` by `_value`
    /// @dev you may need to call `token.approve(bank, _value)`
    /// @param _to account that will have their balance increased by `_value`
    /// @param _value amount of tokens to be transfered
    function depositTokens(address _to, uint256 _value) external;

    /// @notice `value` tokens were transfered from the bank to `to`
    /// @notice the balance of `from` was decreased by `value`
    /// @dev is triggered on any successful call to `transferTokens`
    /// @param from the account/contract that called `transferTokens` and
    ///              got their balance decreased by `value`
    /// @param to the one that received `value` tokens from the bank
    /// @param value amount of tokens that were transfered
    event Transfer(address indexed from, address to, uint256 value);

    /// @notice `value` tokens were transfered from `from` to bank
    /// @notice the balance of `to` was increased by `value`
    /// @dev is triggered on any successful call to `depositTokens`
    /// @param from the account/contract that called `depositTokens` and
    ///              transfered `value` tokens to the bank
    /// @param to the one that got their balance increased by `value`
    /// @param value amount of tokens that were transfered
    event Deposit(address from, address indexed to, uint256 value);
}
