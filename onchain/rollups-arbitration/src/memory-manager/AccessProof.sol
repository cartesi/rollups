// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "utils/Memory.sol";
import { Word } from "utils/Word.sol";

library AccessProof {

    //
    // Access rolling hash
    //

    type RollingHash is bytes32;

    struct WordAccess {
        RollingHash rollingHash;
        Word.Slot slot;
        bool isRead;
    }

    function eq(RollingHash h1, RollingHash h2) internal pure returns (bool) {
        return RollingHash.unwrap(h1) == RollingHash.unwrap(h2);
    }

    function initialRollingHash() internal pure returns (RollingHash) {
        return RollingHash.wrap(bytes32(0x0));
    }

    function nextRollingHash(
        RollingHash rollingHash,
        Word.Slot calldata slot,
        bool isRead
    ) internal pure returns(RollingHash) {
        bytes32 hash = keccak256(
            abi.encode(WordAccess(rollingHash, slot, isRead))
        );

        return RollingHash.wrap(hash);
    }


    //
    // Bit array of 256 bits
    //

    type Bit256Array is uint256; // TODO is this enough?

    function isOne(Bit256Array array, uint index) internal pure returns (bool) {
        require(index < 256, "Bit256Array index out of bounds");

        uint256 a = Bit256Array.unwrap(array);
        return ((a >> index) & 1) == 1;
    }
}
