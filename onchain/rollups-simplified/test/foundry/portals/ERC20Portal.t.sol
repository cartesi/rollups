// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-20 Portal Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {ERC20Portal} from "contracts/portals/ERC20Portal.sol";
import {IERC20Portal} from "contracts/portals/IERC20Portal.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputHeaders} from "contracts/common/InputHeaders.sol";

contract NormalToken is ERC20 {
    constructor(uint256 initialSupply) ERC20("NormalToken", "NORMAL") {
        _mint(msg.sender, initialSupply);
    }
}

contract UntransferableToken is ERC20 {
    constructor(uint256 initialSupply) ERC20("UntransferableToken", "UTFAB") {
        _mint(msg.sender, initialSupply);
    }

    function transfer(address, uint256) public pure override returns (bool) {
        return false;
    }

    function transferFrom(
        address,
        address,
        uint256
    ) public pure override returns (bool) {
        return false;
    }
}

contract ERC20PortalTest is Test {
    InputBox inputBox;
    IERC20Portal erc20Portal;
    IERC20 token;
    address alice;
    address dapp;

    event InputAdded(address indexed dapp, address sender, bytes input);

    function setUp() public {
        inputBox = new InputBox();
        erc20Portal = new ERC20Portal(inputBox);
        alice = address(0xdeadbeef);
        dapp = address(0x12345678);
    }

    function testERC20DepositTrue(uint256 amount, bytes calldata data) public {
        // Create a normal token
        token = new NormalToken(10000);

        // Check if `amount` doesn't surpass the total token supply
        vm.assume(amount <= token.totalSupply());

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            InputHeaders.ERC20_DEPOSIT_TRUE,
            token,
            alice,
            amount,
            data
        );

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Allow the portal to withdraw `amount` tokens from Alice
        token.approve(address(erc20Portal), amount);

        // Save the ERC-20 token balances
        uint256 alicesBalanceBefore = token.balanceOf(alice);
        uint256 dappsBalanceBefore = token.balanceOf(dapp);
        uint256 portalsBalanceBefore = token.balanceOf(address(erc20Portal));

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, address(erc20Portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), alicesBalanceBefore - amount);
        assertEq(token.balanceOf(dapp), dappsBalanceBefore + amount);
        assertEq(token.balanceOf(address(erc20Portal)), portalsBalanceBefore);

        // Check the DApp's input box
        inputBox.inputBoxes(dapp, 0);
        vm.expectRevert();
        inputBox.inputBoxes(dapp, 1);
    }

    function testERC20DepositFalse(uint256 amount, bytes calldata data) public {
        // Create untransferable token
        token = new UntransferableToken(10000);

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            InputHeaders.ERC20_DEPOSIT_FALSE,
            token,
            alice,
            amount,
            data
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Save the ERC-20 token balances
        uint256 alicesBalanceBefore = token.balanceOf(alice);
        uint256 dappsBalanceBefore = token.balanceOf(dapp);
        uint256 portalsBalanceBefore = token.balanceOf(address(erc20Portal));

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, address(erc20Portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), alicesBalanceBefore);
        assertEq(token.balanceOf(dapp), dappsBalanceBefore);
        assertEq(token.balanceOf(address(erc20Portal)), portalsBalanceBefore);

        // Check the DApp's input box
        inputBox.inputBoxes(dapp, 0);
        vm.expectRevert();
        inputBox.inputBoxes(dapp, 1);
    }

    function testRevertsInsufficientAllowance(
        uint256 amount,
        bytes calldata data
    ) public {
        // Create a normal token
        token = new NormalToken(10000);

        // Check if `amount` doesn't surpass the total token supply
        vm.assume(amount <= token.totalSupply());

        // Check if `amount` is non-zero
        vm.assume(amount > 0);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: insufficient allowance");
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the DApp's input box
        vm.expectRevert();
        inputBox.inputBoxes(dapp, 0);
    }

    function testRevertsInsufficientBalance(uint256 amount, bytes calldata data)
        public
    {
        // Create a normal token
        token = new NormalToken(10000);

        // Check if `amount` doesn't surpass the total token supply
        vm.assume(amount <= token.totalSupply());

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Allow the portal to withdraw `amount+1` tokens from Alice
        token.approve(address(erc20Portal), amount + 1);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: transfer amount exceeds balance");
        erc20Portal.depositERC20Tokens(token, dapp, amount + 1, data);

        // Check the DApp's input box
        vm.expectRevert();
        inputBox.inputBoxes(dapp, 0);
    }
}
