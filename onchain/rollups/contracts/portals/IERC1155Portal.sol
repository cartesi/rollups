// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Portal
pragma solidity ^0.8.13;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IPortal} from "./IPortal.sol";

interface IERC1155Portal is IPortal {
    // Permissionless functions

    /// @notice Transfer an ERC-1155 token to a DApp and add an input to
    ///         the DApp's input box to signal such operation.
    /// @param _token The ERC-1155 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenId The identifier of the NFT being transferred
    /// @param _value   Transfer amount
    /// @param _L1data  Additional data with no specified format, MUST be sent unaltered in call to `onERC1155Received` on `_to`
    /// @param _L2data Additional data to be interpreted by L2
    /// @dev The caller must allow the portal to withdraw the token
    ///      from their account beforehand.
    function depositERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256 _tokenId,
        uint256 _value,
        bytes calldata _L1data,
        bytes calldata _L2data
    ) external;

    /// @notice Transfer a batch of ERC-1155 tokens to a DApp and add an input to
    ///         the DApp's input box to signal such operation.
    /// @param _token The ERC-1155 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenIds The identifiers of the tokens being transferred
    /// @param _values Transfer amounts per token type (order and length must match _ids array)
    /// @param _L1data Additional data with no specified format, MUST be sent unaltered in call to the `ERC1155TokenReceiver` hook(s) on `_to`
    /// @param _L2data Additional data to be interpreted by L2
    /// @dev Requirements:
    // `ids` and `amounts` must have the same length.
    //  If `to` refers to a smart contract, it must implement {IERC1155Receiver-onERC1155BatchReceived} and return the
    //  acceptance magic value.
    function depositBatchERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256[] calldata _tokenIds,
        uint256[] calldata _values,
        bytes calldata _L1data,
        bytes calldata _L2data
    ) external;
}
