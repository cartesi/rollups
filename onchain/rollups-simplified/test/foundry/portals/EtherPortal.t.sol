// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Ether Portal Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {EtherPortal} from "contracts/portals/EtherPortal.sol";
import {IEtherPortal} from "contracts/portals/IEtherPortal.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputHeaders} from "contracts/common/InputHeaders.sol";

contract BadEtherReceiver {
    fallback() external payable {
        revert("This contract does not accept Ether");
    }
}

contract EtherPortalTest is Test {
    IInputBox inputBox;
    IEtherPortal etherPortal;
    address alice;
    address dapp;

    event InputAdded(address indexed dapp, address sender, bytes input);

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
        bytes memory input = abi.encodePacked(
            InputHeaders.ETH_DEPOSIT,
            alice,
            value,
            data
        );

        // Transfer Ether to Alice and start impersonating her
        startHoax(alice, value);

        // Save the Ether balances
        uint256 alicesBalanceBefore = alice.balance;
        uint256 dappsBalanceBefore = dapp.balance;
        uint256 portalsBalanceBefore = address(etherPortal).balance;

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, address(etherPortal), input);

        // Deposit Ether in the DApp via the portal
        etherPortal.depositEther{value: value}(dapp, data);

        // Check the balances after the deposit
        assertEq(alice.balance, alicesBalanceBefore - value);
        assertEq(dapp.balance, dappsBalanceBefore + value);
        assertEq(address(etherPortal).balance, portalsBalanceBefore);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsFailedTransfer(uint256 value, bytes calldata data)
        public
    {
        // Create a contract that reverts when it receives Ether
        BadEtherReceiver badEtherReceiver = new BadEtherReceiver();

        startHoax(alice, value);

        // Expect the deposit to revert with the following message
        vm.expectRevert("EtherPortal: transfer failed");
        etherPortal.depositEther{value: value}(address(badEtherReceiver), data);
    }
}
