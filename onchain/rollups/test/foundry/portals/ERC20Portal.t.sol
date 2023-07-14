// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-20 Portal Test
pragma solidity ^0.8.8;

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

    constructor(
        IInputBox _inputBox,
        uint256 _initialSupply
    ) ERC20("WatcherToken", "WTCHR") {
        inputBox = _inputBox;
        _mint(msg.sender, _initialSupply);
    }

    function transfer(
        address _to,
        uint256 _amount
    ) public override returns (bool) {
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
    IERC20Portal portal;
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
        portal = new ERC20Portal(inputBox);
        alice = vm.addr(1);
        dapp = vm.addr(2);
    }

    function testGetInputBox() public {
        assertEq(address(portal.getInputBox()), address(inputBox));
    }

    function testERC20DepositTrue(
        uint256 _amount,
        bytes calldata _data
    ) public {
        // Create a normal token
        token = new NormalToken(_amount);

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            true,
            token,
            alice,
            _amount,
            _data
        );

        // Transfer ERC-20 tokens to Alice and start impersonating her
        require(token.transfer(alice, _amount), "token transfer fail");
        vm.startPrank(alice);

        // Allow the portal to withdraw `_amount` tokens from Alice
        token.approve(address(portal), _amount);

        // Save the ERC-20 token balances
        uint256 aliceBalanceBefore = token.balanceOf(alice);
        uint256 dappBalanceBefore = token.balanceOf(dapp);
        uint256 portalBalanceBefore = token.balanceOf(address(portal));

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        portal.depositERC20Tokens(token, dapp, _amount, _data);
        vm.stopPrank();

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), aliceBalanceBefore - _amount);
        assertEq(token.balanceOf(dapp), dappBalanceBefore + _amount);
        assertEq(token.balanceOf(address(portal)), portalBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testERC20DepositFalse(
        uint256 _amount,
        bytes calldata _data
    ) public {
        // Create untransferable token
        token = new UntransferableToken(_amount);

        // Construct the ERC-20 deposit input
        bytes memory input = abi.encodePacked(
            false,
            token,
            alice,
            _amount,
            _data
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Save the ERC-20 token balances
        uint256 aliceBalanceBefore = token.balanceOf(alice);
        uint256 dappBalanceBefore = token.balanceOf(dapp);
        uint256 portalBalanceBefore = token.balanceOf(address(portal));

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(portal), input);

        // Transfer ERC-20 tokens to the DApp via the portal
        portal.depositERC20Tokens(token, dapp, _amount, _data);
        vm.stopPrank();

        // Check the balances after the deposit
        assertEq(token.balanceOf(alice), aliceBalanceBefore);
        assertEq(token.balanceOf(dapp), dappBalanceBefore);
        assertEq(token.balanceOf(address(portal)), portalBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsInsufficientAllowance(
        uint256 _amount,
        bytes calldata _data
    ) public {
        // Anyone can transfer 0 tokens :-)
        vm.assume(_amount > 0);

        // Create a normal token
        token = new NormalToken(_amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        require(token.transfer(alice, _amount), "token transfer fail");
        vm.startPrank(alice);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: insufficient allowance");
        portal.depositERC20Tokens(token, dapp, _amount, _data);
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testRevertsInsufficientBalance(
        uint256 _amount,
        bytes calldata _data
    ) public {
        // Check if `_amount + 1` won't overflow
        vm.assume(_amount < type(uint256).max);

        // Create a normal token
        token = new NormalToken(_amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        require(token.transfer(alice, _amount), "token transfer fail");
        vm.startPrank(alice);

        // Allow the portal to withdraw `_amount+1` tokens from Alice
        token.approve(address(portal), _amount + 1);

        // Expect deposit to revert with message
        vm.expectRevert("ERC20: transfer amount exceeds balance");
        portal.depositERC20Tokens(token, dapp, _amount + 1, _data);
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testNumberOfInputs(uint256 _amount, bytes calldata _data) public {
        // Create a token that records the number of inputs it has received
        token = new WatcherToken(inputBox, _amount);

        // Transfer ERC-20 tokens to Alice and start impersonating her
        require(token.transfer(alice, _amount), "token transfer fail");
        vm.startPrank(alice);

        // Allow the portal to withdraw `_amount` tokens from Alice
        token.approve(address(portal), _amount);

        // Save number of inputs before the deposit
        uint256 numberOfInputsBefore = inputBox.getNumberOfInputs(dapp);

        // Expect token to be called when no input was added yet
        vm.expectEmit(false, false, false, true, address(token));
        emit WatchedTransfer(
            alice,
            address(dapp),
            _amount,
            numberOfInputsBefore
        );

        // Transfer ERC-20 tokens to DApp
        portal.depositERC20Tokens(token, dapp, _amount, _data);
        vm.stopPrank();

        // Expect new input
        assertEq(inputBox.getNumberOfInputs(dapp), numberOfInputsBefore + 1);
    }
}

contract ERC20PortalHandler is Test {
    IERC20Portal portal;
    IERC20 token;
    IInputBox inputBox;
    address[] public dapps;
    mapping(address => uint256) public dappBalances;
    mapping(address => uint256) public dappNumInputs;

    constructor(IERC20Portal _portal, IERC20 _token) {
        portal = _portal;
        token = _token;
        inputBox = portal.getInputBox();
    }

    function depositERC20Tokens(
        address _dapp,
        uint256 _amount,
        bytes calldata _execLayerData
    ) external {
        address sender = msg.sender;
        if (
            _dapp == address(0) ||
            sender == address(0) ||
            _dapp == address(this)
        ) return;
        _amount = bound(_amount, 0, token.balanceOf(address(this)));

        // fund sender
        require(token.transfer(sender, _amount), "token transfer fail");
        vm.prank(sender);
        token.approve(address(portal), _amount);

        // balance before the deposit
        uint256 senderBalanceBefore = token.balanceOf(sender);
        uint256 dappBalanceBefore = token.balanceOf(_dapp);
        // balance of the portal is 0 all the time during tests
        assertEq(token.balanceOf(address(portal)), 0);

        vm.prank(sender);
        portal.depositERC20Tokens(token, _dapp, _amount, _execLayerData);

        // Check the balances after the deposit
        assertEq(token.balanceOf(sender), senderBalanceBefore - _amount);
        assertEq(token.balanceOf(_dapp), dappBalanceBefore + _amount);
        assertEq(token.balanceOf(address(portal)), 0);

        dapps.push(_dapp);
        dappBalances[_dapp] += _amount;
        assertEq(++dappNumInputs[_dapp], inputBox.getNumberOfInputs(_dapp));
    }

    function getNumDapps() external view returns (uint256) {
        return dapps.length;
    }
}

contract ERC20PortalInvariantTest is Test {
    InputBox inputBox;
    ERC20Portal portal;
    NormalToken token;
    ERC20PortalHandler handler;
    uint256 constant tokenSupply = type(uint256).max;

    function setUp() public {
        inputBox = new InputBox();
        portal = new ERC20Portal(inputBox);
        token = new NormalToken(tokenSupply);
        handler = new ERC20PortalHandler(portal, token);
        // transfer all tokens to handler
        require(
            token.transfer(address(handler), tokenSupply),
            "token transfer fail"
        );

        targetContract(address(handler));
    }

    function invariantTests() external {
        for (uint256 i; i < handler.getNumDapps(); ++i) {
            address dapp = handler.dapps(i);
            assertEq(token.balanceOf(dapp), handler.dappBalances(dapp));
            uint256 numInputs = inputBox.getNumberOfInputs(dapp);
            assertEq(numInputs, handler.dappNumInputs(dapp));
        }
    }
}
