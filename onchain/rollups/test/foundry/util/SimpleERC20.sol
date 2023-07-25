// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title A Simple ERC-20 Contract
pragma solidity ^0.8.8;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract SimpleERC20 is ERC20 {
    constructor(
        address minter,
        uint256 initialSupply
    ) ERC20("SimpleERC20", "SIM20") {
        _mint(minter, initialSupply);
    }
}
