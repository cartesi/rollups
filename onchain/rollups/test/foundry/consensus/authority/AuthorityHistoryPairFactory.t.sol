// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Authority-History Pair Factory Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {AuthorityHistoryPairFactory} from "contracts/consensus/authority/AuthorityHistoryPairFactory.sol";
import {IAuthorityFactory} from "contracts/consensus/authority/IAuthorityFactory.sol";
import {AuthorityFactory} from "contracts/consensus/authority/AuthorityFactory.sol";
import {Authority} from "contracts/consensus/authority/Authority.sol";
import {IHistoryFactory} from "contracts/history/IHistoryFactory.sol";
import {HistoryFactory} from "contracts/history/HistoryFactory.sol";
import {History} from "contracts/history/History.sol";
import {Vm} from "forge-std/Vm.sol";

contract AuthorityFactoryTest is Test {
    AuthorityFactory authorityFactory;
    HistoryFactory historyFactory;
    AuthorityHistoryPairFactory factory;

    event AuthorityHistoryPairFactoryCreated(
        IAuthorityFactory authorityFactory,
        IHistoryFactory historyFactory
    );

    event AuthorityCreated(address authorityOwner, Authority authority);

    event HistoryCreated(address historyOwner, History history);

    function setUp() public {
        authorityFactory = new AuthorityFactory();
        historyFactory = new HistoryFactory();
        factory = new AuthorityHistoryPairFactory(
            authorityFactory,
            historyFactory
        );
    }

    function testFactoryCreation() public {
        vm.recordLogs();

        factory = new AuthorityHistoryPairFactory(
            authorityFactory,
            historyFactory
        );

        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 numOfFactoryCreated;

        for (uint256 i; i < entries.length; ++i) {
            Vm.Log memory entry = entries[i];

            if (
                entry.emitter == address(factory) &&
                entry.topics[0] == AuthorityHistoryPairFactoryCreated.selector
            ) {
                ++numOfFactoryCreated;

                address a;
                address b;

                (a, b) = abi.decode(entry.data, (address, address));

                assertEq(address(authorityFactory), a);
                assertEq(address(historyFactory), b);
            }
        }

        assertEq(numOfFactoryCreated, 1);

        assertEq(
            address(factory.getAuthorityFactory()),
            address(authorityFactory)
        );
        assertEq(address(factory.getHistoryFactory()), address(historyFactory));
    }

    function testNewAuthorityHistoryPair(address _authorityOwner) public {
        vm.assume(_authorityOwner != address(0));

        vm.recordLogs();

        (Authority authority, History history) = factory
            .newAuthorityHistoryPair(_authorityOwner);

        testNewAuthorityHistoryPairAux(_authorityOwner, authority, history);
    }

    function testNewAuthorityHistoryPairAux(
        address _authorityOwner,
        Authority _authority,
        History _history
    ) internal {
        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 numOfAuthorityCreated;
        uint256 numOfHistoryCreated;

        for (uint256 i; i < entries.length; ++i) {
            Vm.Log memory entry = entries[i];

            if (
                entry.emitter == address(authorityFactory) &&
                entry.topics[0] == AuthorityCreated.selector
            ) {
                ++numOfAuthorityCreated;

                address a;
                address b;

                (a, b) = abi.decode(entry.data, (address, address));

                assertEq(address(factory), a);
                assertEq(address(_authority), b);
            }

            if (
                entry.emitter == address(historyFactory) &&
                entry.topics[0] == HistoryCreated.selector
            ) {
                ++numOfHistoryCreated;

                address a;
                address b;

                (a, b) = abi.decode(entry.data, (address, address));

                assertEq(address(_authority), a);
                assertEq(address(_history), b);
            }
        }

        assertEq(numOfAuthorityCreated, 1);
        assertEq(numOfHistoryCreated, 1);

        assertEq(address(_authority.owner()), _authorityOwner);
        assertEq(address(_authority.getHistory()), address(_history));
        assertEq(address(_history.owner()), address(_authority));
    }

    function testNewAuthorityHistoryPairDeterministic(
        address _authorityOwner,
        bytes32 _salt
    ) public {
        vm.assume(_authorityOwner != address(0));

        (address authorityAddress, address historyAddress) = factory
            .calculateAuthorityHistoryAddressPair(_authorityOwner, _salt);

        vm.recordLogs();

        (Authority authority, History history) = factory
            .newAuthorityHistoryPair(_authorityOwner, _salt);

        testNewAuthorityHistoryPairAux(_authorityOwner, authority, history);

        // Precalculated addresses must match actual addresses
        assertEq(authorityAddress, address(authority));
        assertEq(historyAddress, address(history));

        (authorityAddress, historyAddress) = factory
            .calculateAuthorityHistoryAddressPair(_authorityOwner, _salt);

        // Precalculated addresses must STILL match actual addresses
        assertEq(authorityAddress, address(authority));
        assertEq(historyAddress, address(history));

        // Cannot deploy an authority-history pair with the same salt twice
        vm.expectRevert();
        factory.newAuthorityHistoryPair(_authorityOwner, _salt);
    }
}
