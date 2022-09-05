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

    function testRevertsMigrationNotOwner(address alice, address bob) public {
        vm.assume(alice != address(this));
        vm.assume(alice != address(0));
        vm.assume(bob != address(0));
        vm.expectRevert("Ownable: caller is not the owner");
        vm.startPrank(alice);
        history.migrateToConsensus(bob);
        testInitialConsensus(); // consensus hasn't changed
    }

    function testMigrateToZero() public {
        vm.expectRevert("Ownable: new owner is the zero address");
        history.migrateToConsensus(address(0));
        testInitialConsensus(); // consensus hasn't changed
    }

    function testRevertsRenouncingNotOwner(address alice) public {
        vm.assume(alice != address(this));
        vm.assume(alice != address(0));
        vm.expectRevert("Ownable: caller is not the owner");
        vm.startPrank(alice);
        history.renounceOwnership();
        testInitialConsensus(); // consensus hasn't changed
    }

    function isUint128(uint256 n) internal {
        vm.assume(n <= type(uint128).max);
    }

    function submitClaim(
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li,
        uint256 claimIndex
    ) internal {
        isUint128(li);
        vm.assume(fi <= li); // by transitivity, `fi` also fits in a uint128
        vm.expectEmit(false, false, false, true, address(history));
        emit NewClaim(dapp, abi.encode(claimIndex, epochHash, fi, li));
        history.submitClaim(dapp, abi.encode(epochHash, fi, li));
    }

    function testSubmitClaims(
        address dapp,
        bytes32[2] calldata epochHash,
        uint256[2] calldata fi,
        uint256[2] calldata li
    ) public {
        vm.assume(fi[1] > li[0]);
        for (uint256 i; i < 2; ++i) {
            submitClaim(dapp, epochHash[i], fi[i], li[i], i);
        }
    }

    function testRevertsFirstOverflow(
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li
    ) public {
        vm.assume(fi > type(uint128).max); // overflows
        vm.assume(fi <= li);
        vm.expectRevert("SafeCast: value doesn't fit in 128 bits");
        bytes memory data = abi.encode(epochHash, fi, li);
        history.submitClaim(dapp, data);
    }

    function testRevertsLastOverflow(
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li
    ) public {
        isUint128(fi);
        vm.assume(li > type(uint128).max); // overflows
        vm.assume(fi <= li);
        vm.expectRevert("SafeCast: value doesn't fit in 128 bits");
        bytes memory data = abi.encode(epochHash, fi, li);
        history.submitClaim(dapp, data);
    }

    function testRevertsSubmitNotOwner(
        address alice,
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li
    ) public {
        isUint128(fi);
        isUint128(li);
        vm.assume(alice != address(this));
        vm.startPrank(alice);
        vm.assume(fi <= li);
        vm.expectRevert("Ownable: caller is not the owner");
        bytes memory data = abi.encode(epochHash, fi, li);
        history.submitClaim(dapp, data);
    }

    function testRevertsOverlap(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fi1,
        uint256 fi2,
        uint256 li1,
        uint256 li2
    ) public {
        submitClaim(dapp, epochHash1, fi1, li1, 0);
        isUint128(fi2);
        isUint128(li2);
        vm.assume(fi2 <= li2);
        vm.assume(fi2 <= li1); // overlaps with previous claim
        bytes memory data = abi.encode(epochHash2, fi2, li2);
        vm.expectRevert("History: FI <= previous LI");
        history.submitClaim(dapp, data);
    }

    function testRevertsInputIndices(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint256 fi1,
        uint256 fi2,
        uint256 li1,
        uint256 li2
    ) public {
        submitClaim(dapp, epochHash1, fi1, li1, 0);
        isUint128(fi2);
        isUint128(li2);
        vm.assume(fi2 > li2); // starts after it ends
        vm.assume(fi2 > li1);
        bytes memory data = abi.encode(epochHash2, fi2, li2);
        vm.expectRevert("History: FI > LI");
        history.submitClaim(dapp, data);
    }

    function testRevertsSubmitClaimEncoding(address dapp) public {
        bytes memory data = "";
        vm.expectRevert();
        history.submitClaim(dapp, data);
    }

    function checkEpochHash(
        address dapp,
        uint256 claimIndex,
        uint256 inputIndex,
        bytes32 epochHash,
        uint256 epochInputIndex
    ) internal {
        bytes memory data = abi.encode(claimIndex, inputIndex);

        (
            bytes32 retEpochHash,
            uint256 retInputIndex,
            uint256 retEpochInputIndex
        ) = history.getEpochHash(dapp, data);

        assertEq(retEpochHash, epochHash);
        assertEq(retInputIndex, inputIndex);
        assertEq(retEpochInputIndex, epochInputIndex);
    }

    function testGetEpochHash(
        address dapp,
        bytes32[2] calldata epochHash,
        uint256[2] calldata fi,
        uint256[2] calldata li
    ) public {
        testSubmitClaims(dapp, epochHash, fi, li);

        for (uint256 i; i < epochHash.length; ++i) {
            checkEpochHash(dapp, i, fi[i], epochHash[i], 0);
            checkEpochHash(dapp, i, li[i], epochHash[i], li[i] - fi[i]);
            uint256 mi = (fi[i] + li[i]) / 2;
            checkEpochHash(dapp, i, mi, epochHash[i], mi - fi[i]);
        }
    }

    function testRevertsGetEpochHashEncoding(address dapp) public {
        bytes memory data = "";
        vm.expectRevert();
        history.getEpochHash(dapp, data);
    }

    function testRevertsBadInputIndex1(
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li,
        uint256 inputIndex
    ) public {
        submitClaim(dapp, epochHash, fi, li, 0);
        vm.assume(inputIndex > li);
        bytes memory data = abi.encode(0, inputIndex);
        vm.expectRevert("History: bad input index");
        history.getEpochHash(dapp, data);
    }

    function testRevertsBadInputIndex2(
        address dapp,
        bytes32 epochHash,
        uint256 fi,
        uint256 li,
        uint256 inputIndex
    ) public {
        submitClaim(dapp, epochHash, fi, li, 0);
        vm.assume(inputIndex < fi);
        bytes memory data = abi.encode(0, inputIndex);
        vm.expectRevert("History: bad input index");
        history.getEpochHash(dapp, data);
    }

    function submitSeveralClaims(address dapp, bytes32[] calldata epochHash)
        internal
    {
        for (uint256 i; i < epochHash.length; ++i) {
            submitClaim(dapp, epochHash[i], i, i, i);
        }
    }

    function testRevertsBadClaimIndex(
        address dapp,
        bytes32[] calldata epochHash,
        uint256 claimIndex,
        uint256 inputIndex
    ) public {
        submitSeveralClaims(dapp, epochHash);
        vm.assume(claimIndex >= epochHash.length);
        bytes memory data = abi.encode(claimIndex, inputIndex);
        vm.expectRevert(stdError.indexOOBError);
        history.getEpochHash(dapp, data);
    }
}
