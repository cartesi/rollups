// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.17;

import "./Time.sol";

library ArbitrationConstants {
    // maximum tolerance time for participant being censored
    // Time.Duration constant CENSORSHIP_TOLERANCE =
    //     Time.Duration.wrap(60 * 60 * 24 * 7);

    // maximum time for replaying the computation offchain
    // Time.Duration constant VALIDATOR_EFFORT =
    //     Time.Duration.wrap(60 * 60 * 24 * 7); // TODO

    // Dummy
    Time.Duration constant VALIDATOR_EFFORT = Time.Duration.wrap(200);
    Time.Duration constant CENSORSHIP_TOLERANCE = Time.Duration.wrap(90);

    Time.Duration constant DISPUTE_TIMEOUT =
        Time.Duration.wrap(
            Time.Duration.unwrap(CENSORSHIP_TOLERANCE) +
                Time.Duration.unwrap(VALIDATOR_EFFORT)
        );

    // 4-level tournament
    uint64 constant LEVELS = 3;
    // uint64 constant LOG2_MAX_MCYCLE = 63;

    /// @return log2step gap of each leaf in the tournament[level]
    function log2step(uint64 level) internal pure returns (uint64) {
        uint64[LEVELS] memory arr = [
            uint64(31),
            uint64(16),
            uint64(0)
        ];
        return arr[level];
    }

    /// @return height of the tournament[level] tree which is calculated by subtracting the log2step[level] from the log2step[level - 1]
    function height(uint64 level) internal pure returns (uint64) {
        uint64[LEVELS] memory arr = [
            uint64(32),
            uint64(15),
            uint64(16)
        ];
        return arr[level];
    }
}
