// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

/// @title ERC-1155 Single Transfer Portal interface
interface IERC1155SinglePortal is IInputRelay {
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
