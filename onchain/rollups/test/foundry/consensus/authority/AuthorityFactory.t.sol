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
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {Vm} from "forge-std/Vm.sol";

contract AuthorityFactoryTest is Test {
    AuthorityFactory factory;
    InputBox inputBox;

    // event emitted in the factory
    event AuthorityCreated(
        address authorityOwner,
        IInputBox inputBox,
        Authority authority
    );
    // event emitted in the authority contract
    event ConsensusCreated(address owner, IInputBox inputBox);

    function setUp() public {
        factory = new AuthorityFactory();
        inputBox = new InputBox();
    }

    function testNewAuthority(address _authorityOwner) public {
        vm.assume(_authorityOwner != address(0));

        // expect event emitted from the authority contract
        vm.expectEmit(false, false, false, true);
        emit ConsensusCreated(_authorityOwner, inputBox);
        // to check the deployed authority address emitted in the AuthorityCreated event
        // we need to record logs
        vm.recordLogs();

        Authority authority = factory.newAuthority(_authorityOwner, inputBox);

        Vm.Log[] memory entries = vm.getRecordedLogs();

        uint256 count;
        for (uint256 i; i < entries.length; ++i) {
            if (
                entries[i].topics[0] ==
                keccak256("AuthorityCreated(address,address,address)")
            ) {
                // test data
                (
                    address decodedAuthorityOwner,
                    address decodedInputBox,
                    address decodedAuthority
                ) = abi.decode(entries[i].data, (address, address, address));
                assertEq(_authorityOwner, decodedAuthorityOwner);
                assertEq(address(inputBox), decodedInputBox);
                assertEq(address(authority), decodedAuthority);

                ++count;
            }
        }
        // check there's only 1 AuthorityCreated event
        assertEq(count, 1);

        // call to check authority's owner
        assertEq(authority.owner(), _authorityOwner);
    }

    function testNewAuthorityDeterministic(
        address _authorityOwner,
        bytes32 _salt
    ) public {
        vm.assume(_authorityOwner != address(0));

        address precalculatedAddress = factory.calculateAuthorityAddress(
            _authorityOwner,
            inputBox,
            _salt
        );

        // expect event emitted from the authority contract
        vm.expectEmit(false, false, false, true);
        emit ConsensusCreated(_authorityOwner, inputBox);
        // expect event emitted from the factory
        vm.expectEmit(false, false, false, true);
        emit AuthorityCreated(
            _authorityOwner,
            inputBox,
            Authority(precalculatedAddress)
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

        // Cannot deploy an authority with the same salt twice
        vm.expectRevert();
        factory.newAuthority(_authorityOwner, inputBox, _salt);
    }
}
