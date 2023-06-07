// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ENS Resolution Relay Test
pragma solidity ^0.8.8;

import {TestBase} from "../util/TestBase.sol";
import {IENSResolutionRelay} from "contracts/relays/IENSResolutionRelay.sol";
import {Resolver, ENS, ENSResolutionRelay} from "contracts/relays/ENSResolutionRelay.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";

contract ENSResolutionRelayTest is TestBase {
    IInputBox inputBox;
    IENSResolutionRelay relay;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inboxInputIndex,
        address sender,
        bytes input
    );

    function setUp() public {
        inputBox = new InputBox();
        relay = new ENSResolutionRelay(inputBox);
    }

    function testGetInputBox() public {
        assertEq(address(relay.getInputBox()), address(inputBox));
    }

    function testRelayENSResolution(
        address _dapp,
        bytes32 _node,
        address _resolution,
        Resolver _resolver
    ) public isMockable(address(_resolver)) {
        // mock ens and resolver
        address ensAddress = 0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e;
        vm.mockCall(
            ensAddress,
            abi.encodeWithSelector(ENS.resolver.selector, _node),
            abi.encode(_resolver)
        );
        vm.mockCall(
            address(_resolver),
            abi.encodeWithSelector(Resolver.addr.selector, _node),
            abi.encode(_resolution)
        );

        // Check the DApp's input box before
        assertEq(inputBox.getNumberOfInputs(_dapp), 0);

        // Construct the ENS Resolution relay input
        bytes memory input = abi.encodePacked(_node, _resolution);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(_dapp, 0, address(relay), input);

        // Relay the ENS resolution
        relay.relayENSResolution(_dapp, _node);

        // Check the DApp's input box after
        assertEq(inputBox.getNumberOfInputs(_dapp), 1);
    }
}
