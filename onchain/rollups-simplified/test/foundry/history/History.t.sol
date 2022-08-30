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
        vm.assume(fcii <= lcii);
        vm.assume(lcii < type(uint256).max); // otherwise `lcii + 1` would overflow
        bytes memory data = abi.encode(epochHash, fcii, lcii);
        vm.expectEmit(false, false, false, true, address(history));
        emit NewClaim(dapp, data);
        history.submitClaim(dapp, data);
    }

    function testRevertsSubmitNotOwner(
        address alice,
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii
    ) public {
        vm.assume(alice != address(this));
        vm.startPrank(alice);
        vm.assume(fcii <= lcii);
        vm.assume(lcii < type(uint256).max); // otherwise `lcii + 1` would overflow
        vm.expectRevert("Ownable: caller is not the owner");
        bytes memory data = abi.encode(epochHash, fcii, lcii);
        history.submitClaim(dapp, data);
    }

    function testRevertsLowerBound(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fcii1,
        uint256 fcii2,
        uint256 lcii1,
        uint256 lcii2
    ) public {
        testSubmitClaim(dapp, epochHash1, fcii1, lcii1);
        vm.assume(fcii2 <= lcii1); // overlaps with previous claim
        vm.assume(fcii2 <= lcii2);
        vm.assume(lcii2 < type(uint256).max);
        bytes memory data = abi.encode(epochHash2, fcii2, lcii2);
        vm.expectRevert("History: new FCII < IILB");
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
        testSubmitClaim(dapp, epochHash1, fcii1, lcii1);
        vm.assume(fcii2 > lcii1);
        vm.assume(fcii2 > lcii2); // first claim input index is larger than last one
        vm.assume(lcii2 < type(uint256).max);
        bytes memory data = abi.encode(epochHash2, fcii2, lcii2);
        vm.expectRevert("History: new FCII > new LCII");
        history.submitClaim(dapp, data);
    }

    function testRevertsOverflow(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fcii1,
        uint256 fcii2,
        uint256 lcii1
    ) public {
        uint256 lcii2 = type(uint256).max; // lcii2 + 1 overflows
        testSubmitClaim(dapp, epochHash1, fcii1, lcii1);
        vm.assume(fcii2 > lcii1);
        vm.assume(fcii2 <= lcii2);
        bytes memory data = abi.encode(epochHash2, fcii2, lcii2);
        vm.expectRevert(stdError.arithmeticError);
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
        uint256 epochInputIndex
    ) public {
        testSubmitClaim(dapp, epochHash, fcii, lcii);
        vm.assume(epochInputIndex <= lcii - fcii);
        bytes memory data = abi.encode(fcii, epochInputIndex);

        (
            bytes32 retEpochHash,
            uint256 retInputIndex,
            uint256 retEpochInputIndex
        ) = history.getEpochHash(dapp, data);

        assertEq(retEpochHash, epochHash);
        assertEq(retInputIndex, fcii + epochInputIndex);
        assertEq(retEpochInputIndex, epochInputIndex);
    }

    function testRevertsGetEpochHashEncoding(address dapp) public {
        bytes memory data = "";
        vm.expectRevert();
        history.getEpochHash(dapp, data);
    }

    function testFirstInputClaim(address dapp) public {
        bytes memory data = abi.encode(0, 0);

        (
            bytes32 retEpochHash,
            uint256 retInputIndex,
            uint256 retEpochInputIndex
        ) = history.getEpochHash(dapp, data);

        assertEq(retEpochHash, bytes32(0));
        assertEq(retInputIndex, 0);
        assertEq(retEpochInputIndex, 0);
    }

    function testRevertsEpochIndexOverflow(
        address dapp,
        uint256 epochInputIndex
    ) public {
        vm.assume(epochInputIndex < type(uint256).max);
        bytes memory data = abi.encode(
            type(uint256).max - epochInputIndex,
            epochInputIndex + 1
        );
        vm.expectRevert(stdError.arithmeticError);
        history.getEpochHash(dapp, data);
    }

    function testRevertsBadEpochInputIndex(
        address dapp,
        bytes32 epochHash,
        uint256 fcii,
        uint256 lcii,
        uint256 epochInputIndex
    ) public {
        testSubmitClaim(dapp, epochHash, fcii, lcii);
        vm.assume(epochInputIndex > lcii - fcii);
        vm.assume(epochInputIndex <= type(uint256).max - fcii); // to avoid overflows
        bytes memory data = abi.encode(fcii, epochInputIndex);
        vm.expectRevert("History: bad epoch input index");
        history.getEpochHash(dapp, data);
    }
}
