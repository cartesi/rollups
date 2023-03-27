// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Portal Interface
pragma solidity ^0.8.8;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IPortal} from "./IPortal.sol";

interface IERC1155BatchPortal is IPortal {
    // Permissionless functions

    /// @notice Transfer a batch of ERC-1155 tokens to a DApp and add an input to
    ///         the DApp's input box to signal such operation.
    /// @param _token The ERC-1155 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenIds The identifiers of the tokens being transferred
    /// @param _values Transfer amounts per token type
    /// @param _baseLayerData Additional data to be interpreted by the base layer
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    /// @dev Requirements:
    // `ids` and `amounts` must have the same length.
    //  If `to` refers to a smart contract, it must implement {IERC1155Receiver-onERC1155BatchReceived} and return the
    //  acceptance magic value.
    function depositBatchERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256[] calldata _tokenIds,
        uint256[] calldata _values,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external;
}
