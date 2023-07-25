// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title A Simple Consensus Contract
pragma solidity ^0.8.8;

import {AbstractConsensus} from "contracts/consensus/AbstractConsensus.sol";

contract SimpleConsensus is AbstractConsensus {
    function getClaim(
        address,
        bytes calldata
    ) external view returns (bytes32, uint256, uint256) {}
}
