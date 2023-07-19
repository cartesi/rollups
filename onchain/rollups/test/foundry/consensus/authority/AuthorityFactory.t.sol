// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Authority Factory Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {AuthorityFactory} from "contracts/consensus/authority/AuthorityFactory.sol";
import {Authority} from "contracts/consensus/authority/Authority.sol";
import {Vm} from "forge-std/Vm.sol";

contract AuthorityFactoryTest is Test {
    AuthorityFactory factory;

    // event emitted in the factory
    event AuthorityCreated(address authorityOwner, Authority authority);

    function setUp() public {
        factory = new AuthorityFactory();
    }

    function testNewAuthority(address _authorityOwner) public {
        vm.assume(_authorityOwner != address(0));

        vm.recordLogs();

        Authority authority = factory.newAuthority(_authorityOwner);

        testNewAuthorityAux(_authorityOwner, authority);
    }

    function testNewAuthorityAux(
        address _authorityOwner,
        Authority _authority
    ) internal {
        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 numOfAuthorityCreated;

        for (uint256 i; i < entries.length; ++i) {
            Vm.Log memory entry = entries[i];

            if (
                entry.emitter == address(factory) &&
                entry.topics[0] == AuthorityCreated.selector
            ) {
                ++numOfAuthorityCreated;

                address a;
                address b;

                (a, b) = abi.decode(entry.data, (address, address));

                assertEq(_authorityOwner, a);
                assertEq(address(_authority), b);
            }
        }

        assertEq(numOfAuthorityCreated, 1);

        // call to check authority's owner
        assertEq(_authority.owner(), _authorityOwner);
    }

    function testNewAuthorityDeterministic(
        address _authorityOwner,
        bytes32 _salt
    ) public {
        vm.assume(_authorityOwner != address(0));

        address precalculatedAddress = factory.calculateAuthorityAddress(
            _authorityOwner,
            _salt
        );

        vm.recordLogs();

        Authority authority = factory.newAuthority(_authorityOwner, _salt);

        testNewAuthorityAux(_authorityOwner, authority);

        // Precalculated address must match actual address
        assertEq(precalculatedAddress, address(authority));

        precalculatedAddress = factory.calculateAuthorityAddress(
            _authorityOwner,
            _salt
        );

        // Precalculated address must STILL match actual address
        assertEq(precalculatedAddress, address(authority));

        // Cannot deploy an authority with the same salt twice
        vm.expectRevert();
        factory.newAuthority(_authorityOwner, _salt);
    }
}
