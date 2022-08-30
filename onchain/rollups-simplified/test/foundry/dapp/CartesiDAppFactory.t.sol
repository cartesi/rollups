// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Factory Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {CartesiDAppFactory} from "contracts/dapp/CartesiDAppFactory.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {Vm} from "forge-std/Vm.sol";

contract CartesiDAppFactoryTest is Test {
    CartesiDAppFactory factory;

    function setUp() public {
        factory = new CartesiDAppFactory();
    }

    event ApplicationCreated(
        IConsensus indexed consensus,
        address dappOwner,
        bytes32 templateHash,
        CartesiDApp application
    );

    function testNewApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) public {
        vm.assume(_dappOwner != address(0));

        CartesiDApp newDapp = factory.newApplication(
            _consensus,
            _dappOwner,
            _templateHash
        );

        assertEq(address(newDapp.getConsensus()), address(_consensus));
        assertEq(newDapp.owner(), _dappOwner);
        assertEq(newDapp.getTemplateHash(), _templateHash);
    }

    function testApplicationCreatedEvent(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) public {
        vm.assume(_dappOwner != address(0));

        // Start the recorder
        vm.recordLogs();

        // perform call and emit event
        // the first event is `OwnershipTransferred` emitted by Ownable constructor
        // the second event is `OwnershipTransferred` emitted by CartesiDApp constructor
        // the third event is `ApplicationCreated` emitted by `newApplication` function
        // we focus on the third event
        CartesiDApp newDapp = factory.newApplication(
            _consensus,
            _dappOwner,
            _templateHash
        );
        Vm.Log[] memory entries = vm.getRecordedLogs();

        // there are 2 topics
        assertEq(entries[2].topics.length, 2);
        // topics[0] is the event signature
        assertEq(
            entries[2].topics[0],
            keccak256("ApplicationCreated(address,address,bytes32,address)")
        );
        // topics[1] is the IConsensus parameter
        // restrictions on explicit type convertions:
        // "The conversion is only allowed when there is at most one change in sign, width or type-category"
        // ref: https://docs.soliditylang.org/en/latest/080-breaking-changes.html#new-restrictions
        assertEq(
            entries[2].topics[1],
            bytes32(uint256(uint160(address(_consensus))))
        );

        // test data
        // no need to test decodedApplication
        (
            address decodedDappOwner,
            bytes32 decodedTemplateHash,
            address decodedApplication
        ) = abi.decode(entries[2].data, (address, bytes32, address));
        assertEq(_dappOwner, decodedDappOwner);
        assertEq(_templateHash, decodedTemplateHash);
        assertEq(address(newDapp), decodedApplication);
    }
}
