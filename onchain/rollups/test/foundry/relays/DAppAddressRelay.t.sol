// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title DApp Address Relay Test
pragma solidity ^0.8.8;

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
        uint256 indexed inputIndex,
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
        bytes memory input = abi.encode(_dapp);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(_dapp, 0, address(relay), input);

        // Relay the DApp's address
        relay.relayDAppAddress(_dapp);

        // Check the DApp's input box after
        assertEq(inputBox.getNumberOfInputs(_dapp), 1);
    }
}
