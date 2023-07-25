// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
        vm.stopPrank();
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
        vm.stopPrank();
        testInitialConsensus(); // consensus hasn't changed
    }

    function submitClaim(
        address dapp,
        bytes32 epochHash,
        uint128 fi,
        uint128 li
    ) internal {
        vm.expectEmit(true, false, false, true, address(history));
        History.Claim memory claim = History.Claim(epochHash, fi, li);
        emit NewClaimToHistory(dapp, claim);
        bytes memory encodedClaim = abi.encode(dapp, claim);
        history.submitClaim(encodedClaim);
    }

    function testSubmitAndGetClaims(
        address dapp,
        bytes32[3] calldata epochHash,
        uint64[3] calldata indexIncreases
    ) public {
        uint128 fi;
        for (uint256 i; i < epochHash.length; ++i) {
            uint128 li = fi + indexIncreases[i];
            submitClaim(dapp, epochHash[i], fi, li);
            fi = li + 1;
        }
        fi = 0;
        for (uint256 i; i < epochHash.length; ++i) {
            uint128 li = fi + indexIncreases[i];
            checkClaim(dapp, i, epochHash[i], fi, li);
            fi = li + 1;
        }
    }

    function testRevertsSubmitNotOwner(
        address alice,
        address dapp,
        bytes32 epochHash,
        uint128 li
    ) public {
        vm.assume(alice != address(this));
        vm.startPrank(alice);
        vm.expectRevert("Ownable: caller is not the owner");
        history.submitClaim(abi.encode(dapp, epochHash, 0, li));
        vm.stopPrank();
    }

    function testRevertsMaxUint128(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2
    ) public {
        uint128 max = type(uint128).max;
        submitClaim(dapp, epochHash1, 0, max);
        vm.expectRevert(stdError.arithmeticError);
        history.submitClaim(abi.encode(dapp, epochHash2, max, max));
    }

    function testRevertsHeadstart(
        address dapp,
        bytes32 epochHash,
        uint128 fi,
        uint128 li
    ) public {
        vm.assume(fi > 0);
        vm.assume(fi <= li);
        vm.expectRevert(History.UnclaimedInputs.selector);
        history.submitClaim(abi.encode(dapp, epochHash, fi, li));
    }

    function testRevertsOverlap(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint128 fi2,
        uint128 li1,
        uint128 li2
    ) public {
        vm.assume(li1 < type(uint128).max);
        vm.assume(fi2 <= li2);
        vm.assume(fi2 <= li1); // overlaps with previous claim
        submitClaim(dapp, epochHash1, 0, li1);
        vm.expectRevert(History.UnclaimedInputs.selector);
        history.submitClaim(abi.encode(dapp, epochHash2, fi2, li2));
    }

    function testRevertsHole(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint128 fi2,
        uint128 li1,
        uint128 li2
    ) public {
        vm.assume(li1 < type(uint128).max);
        vm.assume(fi2 <= li2);
        vm.assume(fi2 > li1 + 1); // leaves a hole
        submitClaim(dapp, epochHash1, 0, li1);
        vm.expectRevert(History.UnclaimedInputs.selector);
        history.submitClaim(abi.encode(dapp, epochHash2, fi2, li2));
    }

    function testRevertsInputIndices(
        address dapp,
        bytes32 epochHash1,
        bytes32 epochHash2,
        uint128 fi2,
        uint128 li1,
        uint128 li2
    ) public {
        vm.assume(li1 < type(uint128).max);
        vm.assume(fi2 > li2); // starts after it ends
        vm.assume(fi2 > li1);
        submitClaim(dapp, epochHash1, 0, li1);
        vm.expectRevert(History.InvalidInputIndices.selector);
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

        vm.expectRevert(History.InvalidClaimIndex.selector);
        history.getClaim(dapp, abi.encode(claimIndex));
    }
}
