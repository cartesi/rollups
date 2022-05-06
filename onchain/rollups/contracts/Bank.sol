// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Bank contract
pragma solidity ^0.8.0;

import {IBank} from "./IBank.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Bank is IBank {
    IERC20 private immutable token;

    // `balances` maps account/contract addresses to balances
    mapping(address => uint256) private balances;

    constructor(address _token) {
        require(_token != address(0), "Bank: invalid token");
        token = IERC20(_token);
    }

    function getToken() public view override returns (IERC20) {
        return token;
    }

    function balanceOf(address _owner) public view override returns (uint256) {
        return balances[_owner];
    }

    function transferTokens(address _to, uint256 _value) public override {
        // checks
        uint256 balance = balances[msg.sender];
        require(_value <= balance, "Bank: not enough balance");

        // effects
        // Note: this should not underflow because we checked that
        // `_value <= balance` in the `require` above
        unchecked {
            balances[msg.sender] = balance - _value;
        }

        // interactions
        // Note: a well-implemented ERC-20 contract should already
        // require the recipient (in this case, `_to`) to be different
        // than address(0), so we don't need to check it ourselves
        require(token.transfer(_to, _value), "Bank: transfer failed");
        emit Transfer(msg.sender, _to, _value);
    }

    function depositTokens(address _to, uint256 _value) public override {
        // checks
        require(_to != address(0), "Bank: invalid recipient");

        // effects
        // Note: this should not overflow because `IERC20.totalSupply`
        // returns a `uint256` value, so there can't be more than
        // `uint256.max` tokens in an ERC-20 contract.
        balances[_to] += _value;

        // interactions
        // Note: transfers tokens to bank, but emits `Deposit` event
        // with recipient being `_to`
        require(
            token.transferFrom(msg.sender, address(this), _value),
            "Bank: transferFrom failed"
        );
        emit Deposit(msg.sender, _to, _value);
    }
}
