// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Generic ERC1155 Portal facet
pragma solidity ^0.8.0;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

import {IERC1155Portal} from "../interfaces/IERC1155Portal.sol";

import {LibInput} from "../libraries/LibInput.sol";

contract ERC1155PortalFacet is IERC1155Portal {
    using LibInput for LibInput.DiamondStorage;

    bytes32 constant INPUT_HEADER = keccak256("ERC1155_Transfer");

    /// @notice Handle the receipt of an ERC1155 token
    /// @dev The ERC1155 smart contract calls this function on the recipient
    ///  after a `transfer`. This function MAY throw to revert and reject the
    ///  transfer. Return of other than the magic value MUST result in the
    ///  transaction being reverted.
    ///  Note: the contract address is always the message sender.
    /// @param _operator The address which called `safeTransferFrom` function
    /// @param _from The address which previously owned the token
    /// @param _tokenId The token identifier which is being transferred
    /// @param _tokenAmount The token amount which is being transferred
    /// @param _data Additional data to be interpreted by L2
    /// @return this function selector unless throwing
    function onERC1155Received(
        address _operator,
        address _from,
        uint256 _tokenId,
        uint256 _tokenAmount,
        bytes calldata _data
    ) public override returns (bytes4) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        address erc1155Contract = msg.sender;

        bytes memory input = abi.encode(
            INPUT_HEADER,
            erc1155Contract,
            _operator,
            _from,
            _tokenId,
            _tokenAmount,
            _data
        );

        inputDS.addInternalInput(input);

        emit ERC1155Received(
            erc1155Contract,
            _operator,
            _from,
            _tokenId,
            _tokenAmount,
            _data
        );

        // return the magic value to approve the transfer
        return this.onERC1155Received.selector;
    }

    /// @notice Handle the receipt of a batch of ERC1155 tokens
    /// @dev The ERC1155 smart contract calls this function on the recipient
    ///  after a `transfer`. This function MAY throw to revert and reject the
    ///  transfer. Return of other than the magic value MUST result in the
    ///  transaction being reverted.
    ///  Note: the contract address is always the message sender.
    /// @param _operator The address which called `safeBatchTransferFrom` function
    /// @param _from The address which previously owned the tokens
    /// @param _tokenIds The token identifiers which are being transferred
    /// @param _tokenAmounts The token amounts which are being transferred
    /// @param _data Additional data to be interpreted by L2
    /// @return this function selector unless throwing
    function onERC1155BatchReceived(
        address _operator,
        address _from,
        uint256[] calldata _tokenIds,
        uint256[] calldata _tokenAmounts,
        bytes calldata _data
    ) public override returns (bytes4) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        address erc1155Contract = msg.sender;

        bytes memory input = abi.encode(
            INPUT_HEADER,
            erc1155Contract,
            _operator,
            _from,
            _tokenIds,
            _tokenAmounts,
            _data
        );

        inputDS.addInternalInput(input);

        emit ERC1155BatchReceived(
            erc1155Contract,
            _operator,
            _from,
            _tokenIds,
            _tokenAmounts,
            _data
        );

        // return the magic value to approve the transfer
        return this.onERC1155BatchReceived.selector;
    }

    /// @notice withdraw an ERC1155 token from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function erc1155Withdrawal(
        bytes calldata _data
    ) public override returns (bool) {
        // Delegate calls preserve msg.sender, msg.value and address(this)
        require(msg.sender == address(this), "only itself");

        (
            address tokenAddr,
            address payable receiver,
            uint256 tokenId,
            uint256 tokenAmount,
            bytes memory transferData
        ) = abi.decode(_data, (address, address, uint256, uint256, bytes));

        IERC1155 token = IERC1155(tokenAddr);

        // transfer reverts on failure
        token.safeTransferFrom(
            address(this),
            receiver,
            tokenId,
            tokenAmount,
            transferData
        );

        emit ERC1155Withdrawn(
            tokenAddr,
            receiver,
            tokenId,
            tokenAmount,
            transferData
        );
        return true;
    }

    /// @notice withdraw a batch of ERC1155 tokens from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function erc1155BatchWithdrawal(
        bytes calldata _data
    ) public override returns (bool) {
        // Delegate calls preserve msg.sender, msg.value and address(this)
        require(msg.sender == address(this), "only itself");

        (
            address tokenAddr,
            address payable receiver,
            uint256[] memory tokenIds,
            uint256[] memory tokenAmounts,
            bytes memory transferData
        ) = abi.decode(_data, (address, address, uint256[], uint256[], bytes));

        IERC1155 token = IERC1155(tokenAddr);

        // transfer reverts on failure
        token.safeBatchTransferFrom(
            address(this),
            receiver,
            tokenIds,
            tokenAmounts,
            transferData
        );

        emit ERC1155BatchWithdrawn(
            tokenAddr,
            receiver,
            tokenIds,
            tokenAmounts,
            transferData
        );
        return true;
    }
}
