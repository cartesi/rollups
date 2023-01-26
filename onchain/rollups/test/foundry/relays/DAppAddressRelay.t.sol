// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title DApp Address Relay Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {IDAppAddressRelay} from "contracts/relays/IDAppAddressRelay.sol";
import {DAppAddressRelay} from "contracts/relays/DAppAddressRelay.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputEncoding} from "contracts/common/InputEncoding.sol";

contract DAppAddressRelayTest is Test {
    IInputBox inputBox;
    IDAppAddressRelay relay;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inboxInputIndex,
        address sender,
        bytes input
    );

    function setUp() public {
        inputBox = new InputBox();
        relay = new DAppAddressRelay(inputBox);
    }

    function testGetInputBox() public {
        assertEq(address(relay.getInputBox()), address(inputBox));
    }

    function testRelayDAppAddress(address _dapp) public {
        // Check the DApp's input box before
        assertEq(inputBox.getNumberOfInputs(_dapp), 0);

        // Construct the DApp address relay input
        bytes memory input = abi.encodePacked(_dapp);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(_dapp, 0, address(relay), input);

        // Relay the DApp's address
        relay.relayDAppAddress(_dapp);

        // Check the DApp's input box after
        assertEq(inputBox.getNumberOfInputs(_dapp), 1);
    }
}
