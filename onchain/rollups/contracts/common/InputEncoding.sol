// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

/// @title Input Encoding Library

/// @notice Defines the encoding of inputs added by core trustless and
/// permissionless contracts, such as portals and relays.
library InputEncoding {
    /// @notice Encode an Ether deposit.
    /// @param sender The Ether sender
    /// @param value The amount of Ether being sent in Wei
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input
    function encodeEtherDeposit(
        address sender,
        uint256 value,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        return
            abi.encodePacked(
                sender, //              20B
                value, //               32B
                execLayerData //        arbitrary size
            );
    }

    /// @notice Encode an ERC-20 token deposit.
    /// @param ret The return value of `transferFrom`
    /// @param token The token contract
    /// @param sender The token sender
    /// @param amount The amount of tokens being sent
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input
    function encodeERC20Deposit(
        bool ret,
        IERC20 token,
        address sender,
        uint256 amount,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        return
            abi.encodePacked(
                ret, //                 1B
                token, //               20B
                sender, //              20B
                amount, //              32B
                execLayerData //        arbitrary size
            );
    }

    /// @notice Encode an ERC-721 token deposit.
    /// @param token The token contract
    /// @param sender The token sender
    /// @param tokenId The token identifier
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeERC721Deposit(
        IERC721 token,
        address sender,
        uint256 tokenId,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(baseLayerData, execLayerData);
        return
            abi.encodePacked(
                token, //               20B
                sender, //              20B
                tokenId, //             32B
                data //                 arbitrary size
            );
    }

    /// @notice Encode an ERC-1155 single token deposit.
    /// @param token The ERC-1155 token contract
    /// @param sender The token sender
    /// @param tokenId The identifier of the token being transferred
    /// @param value Transfer amount
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeSingleERC1155Deposit(
        IERC1155 token,
        address sender,
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(baseLayerData, execLayerData);
        return
            abi.encodePacked(
                token, //               20B
                sender, //              20B
                tokenId, //             32B
                value, //               32B
                data //                 arbitrary size
            );
    }

    /// @notice Encode an ERC-1155 batch token deposit.
    /// @param token The ERC-1155 token contract
    /// @param sender The token sender
    /// @param tokenIds The identifiers of the tokens being transferred
    /// @param values Transfer amounts per token type
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeBatchERC1155Deposit(
        IERC1155 token,
        address sender,
        uint256[] calldata tokenIds,
        uint256[] calldata values,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(
            tokenIds,
            values,
            baseLayerData,
            execLayerData
        );
        return
            abi.encodePacked(
                token, //                   20B
                sender, //                  20B
                data //                     arbitrary size
            );
    }

    /// @notice Encode a DApp address relay.
    /// @param dapp The DApp address
    /// @return The encoded input
    function encodeDAppAddressRelay(
        address dapp
    ) internal pure returns (bytes memory) {
        return
            abi.encodePacked(
                dapp //                     20B
            );
    }
}
