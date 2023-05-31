// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp Factory Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {SimpleConsensus} from "../util/SimpleConsensus.sol";
import {CartesiDAppFactory} from "contracts/dapp/CartesiDAppFactory.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {Vm} from "forge-std/Vm.sol";

contract CartesiDAppFactoryTest is Test {
    CartesiDAppFactory factory;
    IConsensus consensus;

    function setUp() public {
        factory = new CartesiDAppFactory();
        consensus = new SimpleConsensus();
    }

    event ApplicationCreated(
        IConsensus indexed consensus,
        address dappOwner,
        bytes32 templateHash,
        CartesiDApp application
    );

    function testNewApplication(
        address _dappOwner,
        bytes32 _templateHash
    ) public {
        vm.assume(_dappOwner != address(0));

        CartesiDApp dapp = factory.newApplication(
            consensus,
            _dappOwner,
            _templateHash
        );

        assertEq(address(dapp.getConsensus()), address(consensus));
        assertEq(dapp.owner(), _dappOwner);
        assertEq(dapp.getTemplateHash(), _templateHash);
    }

    function testNewApplicationDeterministic(
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) public {
        vm.assume(_dappOwner != address(0));

        address precalculatedAddress = factory.calculateApplicationAddress(
            consensus,
            _dappOwner,
            _templateHash,
            _salt
        );

        CartesiDApp dapp = factory.newApplication(
            consensus,
            _dappOwner,
            _templateHash,
            _salt
        );

        // Precalculated address must match actual address
        assertEq(precalculatedAddress, address(dapp));

        assertEq(address(dapp.getConsensus()), address(consensus));
        assertEq(dapp.owner(), _dappOwner);
        assertEq(dapp.getTemplateHash(), _templateHash);

        precalculatedAddress = factory.calculateApplicationAddress(
            consensus,
            _dappOwner,
            _templateHash,
            _salt
        );

        // Precalculated address must STILL match actual address
        assertEq(precalculatedAddress, address(dapp));

        // Cannot deploy a DApp with the same salt twice
        vm.expectRevert(bytes(""));
        factory.newApplication(consensus, _dappOwner, _templateHash, _salt);
    }

    function testApplicationCreatedEvent(
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
        CartesiDApp dapp = factory.newApplication(
            consensus,
            _dappOwner,
            _templateHash
        );

        testApplicationCreatedEventAux(_dappOwner, _templateHash, dapp);
    }

    function testApplicationCreatedEventDeterministic(
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) public {
        vm.assume(_dappOwner != address(0));

        // Start the recorder
        vm.recordLogs();

        // perform call and emit event
        // the first event is `OwnershipTransferred` emitted by Ownable constructor
        // the second event is `OwnershipTransferred` emitted by CartesiDApp constructor
        // the third event is `ApplicationCreated` emitted by `newApplication` function
        // we focus on the third event
        CartesiDApp dapp = factory.newApplication(
            consensus,
            _dappOwner,
            _templateHash,
            _salt
        );

        testApplicationCreatedEventAux(_dappOwner, _templateHash, dapp);
    }

    function testApplicationCreatedEventAux(
        address _dappOwner,
        bytes32 _templateHash,
        CartesiDApp _dapp
    ) internal {
        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 numOfApplicationCreated;

        for (uint256 i; i < entries.length; ++i) {
            if (entries[i].topics[0] == ApplicationCreated.selector) {
                assertEq(entries[i].emitter, address(factory));
                assertEq(
                    entries[i].topics[1],
                    bytes32(uint256(uint160(address(consensus))))
                );
                (
                    address decodedDappOwner,
                    bytes32 decodedTemplateHash,
                    address decodedApplication
                ) = abi.decode(entries[i].data, (address, bytes32, address));

                assertEq(_dappOwner, decodedDappOwner);
                assertEq(_templateHash, decodedTemplateHash);
                assertEq(address(_dapp), decodedApplication);

                ++numOfApplicationCreated;
            }
        }

        // exactly one `ApplicationCreated` event must have been emitted
        assertEq(numOfApplicationCreated, 1);
    }
}
