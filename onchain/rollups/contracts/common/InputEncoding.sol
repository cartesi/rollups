// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Encoding Library
pragma solidity ^0.8.13;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

library InputEncoding {
    /// @notice ETH deposit
    bytes1 constant ETH_DEPOSIT = bytes1(0x00);

    /// @notice ERC-20 token deposit where `transferFrom` returns `true`
    bytes1 constant ERC20_DEPOSIT_TRUE = bytes1(0x01);

    /// @notice ERC-20 token deposit where `transferFrom` returns `false`
    bytes1 constant ERC20_DEPOSIT_FALSE = bytes1(0x02);

    /// @notice ERC-721 token deposit
    bytes1 constant ERC721_DEPOSIT = bytes1(0x03);

    /// @notice DApp address
    bytes1 constant DAPP_ADDRESS_RELAY = bytes1(0x10);

    /// @notice Encode Ether deposit
    /// @param sender The Ether sender
    /// @param value The amount of Ether being sent in Wei
    /// @param L2data Additional data to be interpreted by L2
    /// @return The encoded input
    function encodeEtherDeposit(
        address sender,
        uint256 value,
        bytes calldata L2data
    ) internal pure returns (bytes memory) {
        return
            abi.encodePacked(
                ETH_DEPOSIT, // 1B
                sender, //      20B
                value, //       32B
                L2data //       arbitrary size
            );
    }

    /// @notice Encode ERC-20 token deposit
    /// @param ret The return value of `transferFrom`
    /// @param token The token contract
    /// @param sender The token sender
    /// @param amount The amount of tokens being sent
    /// @param L2data Additional data to be interpreted by L2
    /// @return The encoded input
    function encodeERC20Deposit(
        bool ret,
        IERC20 token,
        address sender,
        uint256 amount,
        bytes calldata L2data
    ) internal pure returns (bytes memory) {
        bytes1 header = ret ? ERC20_DEPOSIT_TRUE : ERC20_DEPOSIT_FALSE;
        return
            abi.encodePacked(
                header, // 1B
                token, //  20B
                sender, // 20B
                amount, // 32B
                L2data //  arbitrary size
            );
    }

    /// @notice Encode ERC-721 token deposit
    /// @param token The token contract
    /// @param sender The token sender
    /// @param tokenId The token identifier
    /// @param L1data Additional data to be interpreted by L1
    /// @param L2data Additional data to be interpreted by L2
    /// @return The encoded input
    /// @dev L1data should be forwarded to the ERC-721 token contract
    function encodeERC721Deposit(
        IERC721 token,
        address sender,
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) internal pure returns (bytes memory) {
        bytes memory L1L2data = abi.encode(L1data, L2data);
        return
            abi.encodePacked(
                ERC721_DEPOSIT, // 1B
                token, //          20B
                sender, //         20B
                tokenId, //        32B
                L1L2data //        arbitrary size
            );
    }

    function encodeDAppAddressRelay(
        address dapp
    ) internal pure returns (bytes memory) {
        return
            abi.encodePacked(
                DAPP_ADDRESS_RELAY, // 1B
                dapp //                20B
            );
    }
}
