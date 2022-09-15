// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

library Memory {
    type Address is uint64;
    type Log2Size is uint64;


    function uint64_of_size(Log2Size s) internal pure returns (uint64) {
        return Log2Size.unwrap(s);
    }

    function wordLog2Size() internal pure returns (Log2Size) {
        // Word has 8 bytes.
        return Log2Size.wrap(3);
    }

    function hashLog2Size() internal pure returns (Log2Size) {
        // Hash has 32 bytes.
        return Log2Size.wrap(5);
    }

    function machineLog2Size() internal pure returns (Log2Size) {
        // Machine has 2^64 bytes.
        return Log2Size.wrap(64);
    }

    function outputsLog2Size() internal pure returns (Log2Size) {
        // Outputs has 2^32 leaves, each leave has 32 bytes.
        return Log2Size.wrap(37);
    }
}
