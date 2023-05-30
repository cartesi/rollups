// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

/// @title ERC-721 Portal interface
interface IERC721Portal is IInputRelay {
    // Permissionless functions

    /// @notice Transfer an ERC-721 token to a DApp and add an input to
    /// the DApp's input box to signal such operation.
    ///
    /// The caller must change the approved address for the ERC-721 token
    /// to the portal address beforehand, by calling the `approve` function in the
    /// token contract.
    ///
    /// @param _token The ERC-721 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenId The identifier of the token being transferred
    /// @param _baseLayerData Additional data to be interpreted by the base layer
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    function depositERC721Token(
        IERC721 _token,
        address _dapp,
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external;
}
