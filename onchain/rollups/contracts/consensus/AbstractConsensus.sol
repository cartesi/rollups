// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IConsensus} from "./IConsensus.sol";

/// @title Abstract Consensus
/// @notice An abstract contract that partially implements `IConsensus`.
abstract contract AbstractConsensus is IConsensus {
    /// @notice Emits an `ApplicationJoined` event with the message sender.
    function join() external override {
        emit ApplicationJoined(msg.sender);
    }
}
