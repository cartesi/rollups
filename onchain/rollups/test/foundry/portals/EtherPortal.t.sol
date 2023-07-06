// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title Ether Portal Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {EtherPortal} from "contracts/portals/EtherPortal.sol";
import {IEtherPortal} from "contracts/portals/IEtherPortal.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputEncoding} from "contracts/common/InputEncoding.sol";

contract BadEtherReceiver {
    receive() external payable {
        revert("This contract does not accept Ether");
    }
}

contract EtherReceiver {
    receive() external payable {}
}

contract InputBoxWatcher {
    IInputBox inputBox;

    event WatchedFallback(
        address sender,
        uint256 value,
        uint256 numberOfInputs
    );

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    receive() external payable {
        uint256 numberOfInputs = inputBox.getNumberOfInputs(address(this));
        emit WatchedFallback(msg.sender, msg.value, numberOfInputs);
    }
}

contract EtherPortalTest is Test {
    IInputBox inputBox;
    IEtherPortal etherPortal;
    address alice;
    address dapp;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inputIndex,
        address sender,
        bytes input
    );
    event WatchedFallback(
        address sender,
        uint256 value,
        uint256 numberOfInputs
    );

    function setUp() public {
        inputBox = new InputBox();
        etherPortal = new EtherPortal(inputBox);
        alice = address(0xdeadbeef);
        dapp = address(0x12345678);
    }

    function testGetInputBox() public {
        assertEq(address(etherPortal.getInputBox()), address(inputBox));
    }

    function testEtherDeposit(uint256 value, bytes calldata data) public {
        // Construct the Ether deposit input
        bytes memory input = abi.encode(alice, value, data);

        // Transfer Ether to Alice and start impersonating her
        startHoax(alice, value);

        // Save the Ether balances
        uint256 alicesBalanceBefore = alice.balance;
        uint256 dappsBalanceBefore = dapp.balance;
        uint256 portalsBalanceBefore = address(etherPortal).balance;

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(etherPortal), input);

        // Deposit Ether in the DApp via the portal
        etherPortal.depositEther{value: value}(dapp, data);

        // Check the balances after the deposit
        assertEq(alice.balance, alicesBalanceBefore - value);
        assertEq(dapp.balance, dappsBalanceBefore + value);
        assertEq(address(etherPortal).balance, portalsBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsFailedTransfer(
        uint256 value,
        bytes calldata data
    ) public {
        // Create a contract that reverts when it receives Ether
        BadEtherReceiver badEtherReceiver = new BadEtherReceiver();

        startHoax(alice, value);

        // Expect the deposit to revert with the following message
        vm.expectRevert(EtherPortal.EtherTransferFailed.selector);
        etherPortal.depositEther{value: value}(address(badEtherReceiver), data);
    }

    function testNumberOfInputs(uint256 value, bytes calldata data) public {
        // Create a contract that records the number of inputs it has received
        InputBoxWatcher watcher = new InputBoxWatcher(inputBox);

        startHoax(alice, value);

        // Expect new contract to have no inputs yet
        uint256 numberOfInputsBefore = inputBox.getNumberOfInputs(
            address(watcher)
        );

        // Expect WatchedFallback to be emitted
        vm.expectEmit(false, false, false, true, address(watcher));
        emit WatchedFallback(address(etherPortal), value, numberOfInputsBefore);

        // Transfer Ether to contract
        etherPortal.depositEther{value: value}(address(watcher), data);

        // Expect new input
        assertEq(
            inputBox.getNumberOfInputs(address(watcher)),
            numberOfInputsBefore + 1
        );
    }
}

contract EtherPortalHandler is Test {
    IEtherPortal portal;
    IInputBox inputBox;
    address[] dapps;
    mapping(address => uint256) public dappBalances;
    mapping(address => uint256) public dappNumInputs;

    constructor(IEtherPortal _portal, address[] memory _dapps) {
        portal = _portal;
        inputBox = portal.getInputBox();
        dapps = _dapps;
    }

    function depositEther(
        uint256 _dappIndex,
        uint256 _amount,
        bytes calldata _execLayerData
    ) external {
        address sender = msg.sender;
        address dapp = dapps[_dappIndex % dapps.length];
        _amount = bound(_amount, 0, type(uint128).max);

        // fund sender
        for (uint256 i; i < dapps.length; ++i) {
            if (sender == dapps[i]) {
                return;
            }
        }
        vm.deal(sender, _amount);

        // balance before the deposit
        uint256 senderBalanceBefore = sender.balance;
        uint256 dappBalanceBefore = dapp.balance;
        // balance of the portal is 0 all the time during tests
        assertEq(address(portal).balance, 0);

        vm.prank(sender);
        portal.depositEther{value: _amount}(dapp, _execLayerData);

        // Check the balances after the deposit
        assertEq(sender.balance, senderBalanceBefore - _amount);
        assertEq(dapp.balance, dappBalanceBefore + _amount);
        assertEq(address(portal).balance, 0);

        dappBalances[dapp] += _amount;
        assertEq(++dappNumInputs[dapp], inputBox.getNumberOfInputs(dapp));
    }
}

contract EtherPortalInvariantTest is Test {
    InputBox inputBox;
    EtherPortal portal;
    EtherPortalHandler handler;
    uint256 numDapps;
    address[] dapps;

    function setUp() public {
        inputBox = new InputBox();
        portal = new EtherPortal(inputBox);
        numDapps = 30;
        for (uint256 i; i < numDapps; ++i) {
            dapps.push(address(new EtherReceiver()));
        }
        handler = new EtherPortalHandler(portal, dapps);

        targetContract(address(handler));
    }

    function invariantTests() external {
        for (uint256 i; i < numDapps; ++i) {
            address dapp = dapps[i];
            assertEq(dapp.balance, handler.dappBalances(dapp));
            uint256 numInputs = inputBox.getNumberOfInputs(dapp);
            assertEq(numInputs, handler.dappNumInputs(dapp));
        }
    }
}
