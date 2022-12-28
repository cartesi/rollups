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
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputEncoding} from "contracts/common/InputEncoding.sol";

contract NormalToken is ERC20 {
    constructor(uint256 _initialSupply) ERC20("NormalToken", "NORMAL") {
        _mint(msg.sender, _initialSupply);
    }
}

contract UntransferableToken is ERC20 {
    constructor(uint256 _initialSupply) ERC20("UntransferableToken", "UTFAB") {
        _mint(msg.sender, _initialSupply);
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

contract WatcherToken is ERC20 {
    IInputBox inputBox;

    event WatchedTransfer(
        address from,
        address to,
        uint256 amount,
        uint256 numberOfInputs
    );

    constructor(IInputBox _inputBox, uint256 _initialSupply)
        ERC20("WatcherToken", "WTCHR")
    {
        inputBox = _inputBox;
        _mint(msg.sender, _initialSupply);
    }

    function transfer(address _to, uint256 _amount)
        public
        override
        returns (bool)
    {
        emit WatchedTransfer(
            msg.sender,
            _to,
            _amount,
            inputBox.getNumberOfInputs(_to)
        );
        return super.transfer(_to, _amount);
    }

    function transferFrom(
        address _from,
        address _to,
        uint256 _amount
    ) public override returns (bool) {
        emit WatchedTransfer(
            _from,
            _to,
            _amount,
            inputBox.getNumberOfInputs(_to)
        );
        return super.transferFrom(_from, _to, _amount);
    }
}

contract ERC20PortalTest is Test {
    IInputBox inputBox;
    IERC20Portal erc20Portal;
    IERC20 token;
    address alice;
    address dapp;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inputIndex,
        address sender,
        bytes input
    );
    event WatchedTransfer(
        address from,
        address to,
        uint256 amount,
        uint256 numberOfInputs
    );

    function setUp() public {
        inputBox = new InputBox();
        erc20Portal = new ERC20Portal(inputBox);
        alice = address(0xdeadbeef);
        dapp = address(0x12345678);
    }

    function testGetInputBox() public {
        assertEq(address(erc20Portal.getInputBox()), address(inputBox));
    }

    function testERC20DepositTrue(uint256 amount, bytes calldata data) public {
        // Create a normal token
        token = new NormalToken(amount);

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            InputEncoding.ERC20_DEPOSIT_TRUE,
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
        emit InputAdded(dapp, 0, address(erc20Portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), alicesBalanceBefore - amount);
        assertEq(token.balanceOf(dapp), dappsBalanceBefore + amount);
        assertEq(token.balanceOf(address(erc20Portal)), portalsBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testERC20DepositFalse(uint256 amount, bytes calldata data) public {
        // Create untransferable token
        token = new UntransferableToken(amount);

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            InputEncoding.ERC20_DEPOSIT_FALSE,
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
        emit InputAdded(dapp, 0, address(erc20Portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), alicesBalanceBefore);
        assertEq(token.balanceOf(dapp), dappsBalanceBefore);
        assertEq(token.balanceOf(address(erc20Portal)), portalsBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsInsufficientAllowance(
        uint256 amount,
        bytes calldata data
    ) public {
        // Anyone can transfer 0 tokens :-)
        vm.assume(amount > 0);

        // Create a normal token
        token = new NormalToken(amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: insufficient allowance");
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testRevertsInsufficientBalance(uint256 amount, bytes calldata data)
        public
    {
        // Check if `amount + 1` won't overflow
        vm.assume(amount < type(uint256).max);

        // Create a normal token
        token = new NormalToken(amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Allow the portal to withdraw `amount+1` tokens from Alice
        token.approve(address(erc20Portal), amount + 1);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: transfer amount exceeds balance");
        erc20Portal.depositERC20Tokens(token, dapp, amount + 1, data);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testNumberOfInputs(uint256 amount, bytes calldata data) public {
        // Create a token that records the number of inputs it has received
        token = new WatcherToken(inputBox, amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        token.transfer(alice, amount);
        vm.startPrank(alice);

        // Allow the portal to withdraw `amount` tokens from Alice
        token.approve(address(erc20Portal), amount);

        // Save number of inputs before the deposit
        uint256 numberOfInputsBefore = inputBox.getNumberOfInputs(dapp);

        // Expect token to be called when no input was added yet
        vm.expectEmit(false, false, false, true, address(token));
        emit WatchedTransfer(
            alice,
            address(dapp),
            amount,
            numberOfInputsBefore
        );

        // Transfer ERC-20 tokens to DApp
        erc20Portal.depositERC20Tokens(token, dapp, amount, data);

        // Expect new input
        assertEq(inputBox.getNumberOfInputs(dapp), numberOfInputsBefore + 1);
    }
}
