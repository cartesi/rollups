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

import {TestBase} from "../../util/TestBase.sol";
import {AuthorityFactory} from "contracts/consensus/authority/AuthorityFactory.sol";
import {Authority} from "contracts/consensus/authority/Authority.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {Vm} from "forge-std/Vm.sol";

contract AuthorityFactoryTest is TestBase {
    AuthorityFactory factory;
    InputBox inputBox;

    function setUp() public {
        factory = new AuthorityFactory();
        inputBox = new InputBox();
    }

    event AuthorityCreated(
        address authorityOwner,
        InputBox _inputBox,
        Authority authority
    );

    function testNewAuthority(
        address _authorityOwner
    ) public {
        vm.assume(_authorityOwner != address(0));

        Authority authority = factory.newAuthority(
            _authorityOwner,
            inputBox
        );

        assertEq(authority.owner(), _authorityOwner);
    }

    function testNewApplicationDeterministic(
        address _authorityOwner,
        bytes32 _salt
    ) public {
        vm.assume(_authorityOwner != address(0));

        address precalculatedAddress = factory.calculateAuthorityAddress(
            _authorityOwner,
            inputBox,
            _salt
        );

        Authority authority = factory.newAuthority(
            _authorityOwner,
            inputBox,
            _salt
        );

        // Precalculated address must match actual address
        assertEq(precalculatedAddress, address(authority));

        assertEq(authority.owner(), _authorityOwner);

        precalculatedAddress = factory.calculateAuthorityAddress(
            _authorityOwner,
            inputBox,
            _salt
        );

        // Precalculated address must STILL match actual address
        assertEq(precalculatedAddress, address(authority));

        // Cannot deploy a DApp with the same salt twice
        vm.expectRevert(bytes(""));
        factory.newAuthority(
            _authorityOwner,
            inputBox,
            _salt
        );
    }

}
