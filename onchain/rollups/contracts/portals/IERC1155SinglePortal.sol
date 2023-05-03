// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {IPortal} from "./IPortal.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

/// @title ERC-1155 Single Transfer Portal interface
interface IERC1155SinglePortal is IPortal {
    // Permissionless functions

    /// @notice Transfer an ERC-1155 token to a DApp and add an input to
    /// the DApp's input box to signal such operation.
    ///
    /// The caller must enable approval for the portal to manage all of their tokens
    /// beforehand, by calling the `setApprovalForAll` function in the token contract.
    ///
    /// @param _token The ERC-1155 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenId The identifier of the token being transferred
    /// @param _value Transfer amount
    /// @param _baseLayerData Additional data to be interpreted by the base layer
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    function depositSingleERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256 _tokenId,
        uint256 _value,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external;
}
