// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title History Test
pragma solidity ^0.8.8;

import {Test, stdError} from "forge-std/Test.sol";
import {History} from "contracts/history/History.sol";

contract HistoryTest is Test {
    History history;

    event NewClaimToHistory(address indexed dapp, History.Claim claim);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    function setUp() public {
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        history = new History(address(this));
    }

    function testOwner(address owner) public {
        vm.assume(owner != address(0));
        history = new History(owner);
        assertEq(history.owner(), owner);
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
        vm.expectRevert("Ownable: caller is not the owner");
        vm.startPrank(alice);
        history.renounceOwnership();
        testInitialConsensus(); // consensus hasn't changed
    }

    function submitClaim(
        address dapp,
        bytes32 epochHash,
        uint128 fi,
        uint128 li
    ) internal {
        vm.assume(fi <= li);
        vm.expectEmit(false, false, false, true, address(history));
        History.Claim memory claim = History.Claim(epochHash, fi, li);
        emit NewClaimToHistory(dapp, claim);
        bytes memory encodedClaim = abi.encode(dapp, claim);
        history.submitClaim(encodedClaim);
    }

    function testSubmitClaims(
        address dapp,
        bytes32[2] calldata epochHash,
        uint128[2] calldata fi,
        uint128[2] calldata li
    ) public {
        vm.assume(fi[1] > li[0]);
        for (uint256 i; i < 2; ++i) {
            submitClaim(dapp, epochHash[i], fi[i], li[i]);
        }
    }

    function testRevertsSubmitNotOwner(
        address alice,
        address dapp,
        bytes32 epochHash,
        uint128 fi,
        uint128 li
    ) public {
        vm.assume(alice != address(this));
        vm.startPrank(alice);
        vm.assume(fi <= li);
        vm.expectRevert("Ownable: caller is not the owner");
        history.submitClaim(abi.encode(dapp, epochHash, fi, li));
    }

    function testRevertsOverlap(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint128 fi1,
        uint128 fi2,
        uint128 li1,
        uint128 li2
    ) public {
        submitClaim(dapp, epochHash1, fi1, li1);
        vm.assume(fi2 <= li2);
        vm.assume(fi2 <= li1); // overlaps with previous claim
        vm.expectRevert("History: FI <= previous LI");
        history.submitClaim(abi.encode(dapp, epochHash2, fi2, li2));
    }

    function testRevertsInputIndices(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint128 fi1,
        uint128 fi2,
        uint128 li1,
        uint128 li2
    ) public {
        submitClaim(dapp, epochHash1, fi1, li1);
        vm.assume(fi2 > li2); // starts after it ends
        vm.assume(fi2 > li1);
        vm.expectRevert("History: FI > LI");
        history.submitClaim(abi.encode(dapp, epochHash2, fi2, li2));
    }

    function testRevertsSubmitClaimEncoding() public {
        vm.expectRevert();
        history.submitClaim("");
    }

    function checkClaim(
        address dapp,
        uint256 claimIndex,
        bytes32 epochHash,
        uint256 firstInputIndex,
        uint256 lastInputIndex
    ) internal {
        (
            bytes32 retEpochHash,
            uint256 retFirstInputIndex,
            uint256 retLastInputIndex
        ) = history.getClaim(dapp, abi.encode(claimIndex));

        assertEq(retEpochHash, epochHash);
        assertEq(retFirstInputIndex, firstInputIndex);
        assertEq(retLastInputIndex, lastInputIndex);
    }

    function testGetClaim(
        address dapp,
        bytes32[2] calldata epochHash,
        uint128[2] calldata fi,
        uint128[2] calldata li
    ) public {
        testSubmitClaims(dapp, epochHash, fi, li);
        for (uint256 i; i < epochHash.length; ++i) {
            checkClaim(dapp, i, epochHash[i], fi[i], li[i]);
        }
    }

    function testRevertsGetClaimEncoding(address dapp) public {
        vm.expectRevert();
        history.getClaim(dapp, "");
    }

    function testRevertsBadClaimIndex(
        address dapp,
        bytes32[] calldata epochHash,
        uint256 claimIndex
    ) public {
        vm.assume(claimIndex >= epochHash.length);

        // submit several claims with 1 input each
        for (uint128 i; i < epochHash.length; ++i) {
            submitClaim(dapp, epochHash[i], i, i);
        }

        vm.expectRevert(stdError.indexOOBError);
        history.getClaim(dapp, abi.encode(claimIndex));
    }
}
