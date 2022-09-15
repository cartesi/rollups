// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

library Arithmetic {
    function semiSum(uint64 a, uint64 b) external pure returns(uint64) {
        assert(a <= b);
        return a + (b - a) / 2;
    }

    function monus(uint256 a, uint256 b) external pure returns(uint256) {
        if (a < b) {
            return 0;
        } else {
            return a - b;
        }
    }
}
