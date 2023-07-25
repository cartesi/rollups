// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title Cartesi DApp Factory Test
pragma solidity ^0.8.8;

import {TestBase} from "../util/TestBase.sol";
import {SimpleConsensus} from "../util/SimpleConsensus.sol";
import {CartesiDAppFactory} from "contracts/dapp/CartesiDAppFactory.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {Vm} from "forge-std/Vm.sol";

contract CartesiDAppFactoryTest is TestBase {
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

        uint256 numOfApplicationsCreated;

        for (uint256 i; i < entries.length; ++i) {
            Vm.Log memory entry = entries[i];

            if (
                entry.emitter == address(factory) &&
                entry.topics[0] == ApplicationCreated.selector
            ) {
                ++numOfApplicationsCreated;

                assertEq(
                    entry.topics[1],
                    bytes32(uint256(uint160(address(consensus))))
                );

                address a;
                bytes32 b;
                address c;

                (a, b, c) = abi.decode(entry.data, (address, bytes32, address));

                assertEq(_dappOwner, a);
                assertEq(_templateHash, b);
                assertEq(address(_dapp), c);
            }
        }

        assertEq(numOfApplicationsCreated, 1);
    }
}
