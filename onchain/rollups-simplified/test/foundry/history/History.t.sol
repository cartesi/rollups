// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title History Test
pragma solidity ^0.8.13;

import {Test, stdError} from "forge-std/Test.sol";
import {History} from "contracts/history/History.sol";

contract HistoryTest is Test {
    History history;

    event NewClaim(address dapp, bytes data);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    function setUp() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        history = new History();
    }

    function testInitialConsensus() public {
        assertEq(history.owner(), address(this));
    }

    function testMigrateToConsensus(address consensus) public {
        vm.assume(consensus != address(0));
        vm.expectEmit(true, true, false, false, address(history));
        emit OwnershipTransferred(address(this), consensus);
        history.migrateToConsensus(consensus);
        assertEq(history.owner(), consensus);
    }

    function testRenounceOwnership() public {
        vm.expectEmit(true, true, false, false, address(history));
        emit OwnershipTransferred(address(this), address(0));
        history.renounceOwnership();
        assertEq(history.owner(), address(0));
    }

    function testRevertsMigrationNotOwner(address alice) public {
        vm.assume(alice != address(this));
        vm.assume(alice != address(0));
        vm.startPrank(alice);
        vm.expectRevert("Ownable: caller is not the owner");
        history.migrateToConsensus(alice);
        testInitialConsensus(); // consensus hasn't changed
    }

    function testMigrateToZero() public {
        vm.expectRevert("Ownable: new owner is the zero address");
        history.migrateToConsensus(address(0));
        testInitialConsensus(); // consensus hasn't changed
    }

    function testSubmitClaim(
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii
    ) public {
        vm.assume(fcii <= type(uint128).max);
        vm.assume(lcii <= type(uint128).max);
        vm.assume(fcii <= lcii);
        bytes memory data = abi.encode(epochHash, fcii, lcii);
        vm.expectEmit(false, false, false, true, address(history));
        bytes memory eventData = abi.encode(0, epochHash, fcii, lcii);
        emit NewClaim(dapp, eventData);
        history.submitClaim(dapp, data);
    }

    function testRevertsSubmitNotOwner(
        address alice,
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii
    ) public {
        vm.assume(fcii <= type(uint128).max);
        vm.assume(lcii <= type(uint128).max);
        vm.assume(alice != address(this));
        vm.startPrank(alice);
        vm.assume(fcii <= lcii);
        vm.expectRevert("Ownable: caller is not the owner");
        bytes memory data = abi.encode(epochHash, fcii, lcii);
        history.submitClaim(dapp, data);
    }

    function testRevertsOverlap(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fcii1,
        uint256 fcii2,
        uint256 lcii1,
        uint256 lcii2
    ) public {
        vm.assume(fcii1 <= type(uint128).max);
        vm.assume(lcii1 <= type(uint128).max);
        vm.assume(fcii2 <= type(uint128).max);
        vm.assume(lcii2 <= type(uint128).max);
        testSubmitClaim(dapp, epochHash1, fcii1, lcii1);
        vm.assume(fcii2 <= lcii1); // overlaps with previous claim
        vm.assume(fcii2 <= lcii2);
        bytes memory data = abi.encode(epochHash2, fcii2, lcii2);
        vm.expectRevert("History: FI <= previous LI");
        history.submitClaim(dapp, data);
    }

    function testRevertsInputIndices(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fcii1,
        uint256 fcii2,
        uint256 lcii1,
        uint256 lcii2
    ) public {
        vm.assume(fcii1 <= type(uint128).max);
        vm.assume(lcii1 <= type(uint128).max);
        vm.assume(fcii2 <= type(uint128).max);
        vm.assume(lcii2 <= type(uint128).max);
        testSubmitClaim(dapp, epochHash1, fcii1, lcii1);
        vm.assume(fcii2 > lcii1);
        vm.assume(fcii2 > lcii2); // first claim input index is larger than last one
        bytes memory data = abi.encode(epochHash2, fcii2, lcii2);
        vm.expectRevert("History: FI > LI");
        history.submitClaim(dapp, data);
    }

    function testRevertsSubmitClaimEncoding(address dapp) public {
        bytes memory data = "";
        vm.expectRevert();
        history.submitClaim(dapp, data);
    }

    function testGetEpochHash(
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii,
        uint256 inputIndex
    ) public {
        vm.assume(fcii <= type(uint128).max);
        vm.assume(lcii <= type(uint128).max);
        testSubmitClaim(dapp, epochHash, fcii, lcii);
        vm.assume(fcii <= inputIndex && inputIndex <= lcii);
        bytes memory data = abi.encode(0, inputIndex);

        (
            bytes32 retEpochHash,
            uint256 retInputIndex,
            uint256 retEpochInputIndex
        ) = history.getEpochHash(dapp, data);

        assertEq(retEpochHash, epochHash);
        assertEq(retInputIndex, inputIndex);
        assertEq(retEpochInputIndex, inputIndex - fcii);
    }

    function testRevertsGetEpochHashEncoding(address dapp) public {
        bytes memory data = "";
        vm.expectRevert();
        history.getEpochHash(dapp, data);
    }

    function testRevertsBadEpochInputIndex(
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii,
        uint256 inputIndex
    ) public {
        vm.assume(fcii <= type(uint128).max);
        vm.assume(lcii <= type(uint128).max);
        testSubmitClaim(dapp, epochHash, fcii, lcii);
        vm.assume(inputIndex > lcii);
        bytes memory data = abi.encode(0, inputIndex);
        vm.expectRevert("History: bad input index");
        history.getEpochHash(dapp, data);
    }
}
