// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-721 Portal
pragma solidity ^0.8.13;

import {IPortal} from "./IPortal.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

interface IERC721Portal is IPortal {
    // Permissionless functions

    /// @notice Transfer an ERC-721 token to a DApp and add an input to
    ///         the DApp's input box to signal such operation.
    /// @param _token The ERC-721 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenId The identifier of the NFT being transferred
    /// @param _data Additional data to be interpreted by L2
    /// @dev The caller must allow the portal to withdraw the token
    ///      from their account beforehand.
    function depositERC721Token(
        IERC721 _token,
        address _dapp,
        uint256 _tokenId,
        bytes calldata _data
    ) external;
}
