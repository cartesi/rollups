// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title A Simple ERC-721 Contract
pragma solidity ^0.8.8;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract SimpleERC721 is ERC721 {
    constructor(
        address minter,
        uint256 tokenId
    ) ERC721("SimpleERC721", "SIM721") {
        _safeMint(minter, tokenId);
    }
}
