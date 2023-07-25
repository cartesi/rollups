// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

/// @title ERC-1155 Batch Transfer Portal interface
interface IERC1155BatchPortal is IInputRelay {
    // Permissionless functions

    /// @notice Transfer a batch of ERC-1155 tokens to a DApp and add an input to
    /// the DApp's input box to signal such operation.
    ///
    /// The caller must enable approval for the portal to manage all of their tokens
    /// beforehand, by calling the `setApprovalForAll` function in the token contract.
    ///
    /// @param _token The ERC-1155 token contract
    /// @param _dapp The address of the DApp
    /// @param _tokenIds The identifiers of the tokens being transferred
    /// @param _values Transfer amounts per token type
    /// @param _baseLayerData Additional data to be interpreted by the base layer
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    ///
    /// @dev Please make sure `_tokenIds` and `_values` have the same length.
    function depositBatchERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256[] calldata _tokenIds,
        uint256[] calldata _values,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external;
}
