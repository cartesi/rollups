// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Generic ERC721 Portal facet
pragma solidity ^0.8.0;

import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

import {IERC721Portal} from "../interfaces/IERC721Portal.sol";

import {LibInput} from "../libraries/LibInput.sol";

contract ERC721PortalFacet is IERC721Portal {
    using LibInput for LibInput.DiamondStorage;

    bytes32 constant INPUT_HEADER = keccak256("ERC721_Transfer");

    /// @notice Handle the receipt of an NFT
    /// @dev The ERC721 smart contract calls this function on the recipient
    ///  after a `transfer`. This function MAY throw to revert and reject the
    ///  transfer. Return of other than the magic value MUST result in the
    ///  transaction being reverted.
    ///  Note: the contract address is always the message sender.
    /// @param _operator The address which called `safeTransferFrom` function
    /// @param _from The address which previously owned the token
    /// @param _tokenId The NFT identifier which is being transferred
    /// @param _data Additional data to be interpreted by L2
    /// @return this function selector unless throwing
    function onERC721Received(
        address _operator,
        address _from,
        uint256 _tokenId,
        bytes calldata _data
    ) public override returns (bytes4) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        address erc721Contract = msg.sender;

        bytes memory input = abi.encode(
            INPUT_HEADER,
            erc721Contract,
            _operator,
            _from,
            _tokenId,
            _data
        );

        inputDS.addInternalInput(input);

        emit ERC721Received(erc721Contract, _operator, _from, _tokenId, _data);

        // return the magic value to approve the transfer
        return this.onERC721Received.selector;
    }

    /// @notice withdraw an ERC721 token from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function erc721Withdrawal(
        bytes calldata _data
    ) public override returns (bool) {
        // Delegate calls preserve msg.sender, msg.value and address(this)
        require(msg.sender == address(this), "only itself");

        (address tokenAddr, address payable receiver, uint256 tokenId) = abi
            .decode(_data, (address, address, uint256));

        IERC721 token = IERC721(tokenAddr);

        // transfer reverts on failure
        token.safeTransferFrom(address(this), receiver, tokenId);

        emit ERC721Withdrawn(tokenAddr, receiver, tokenId);
        return true;
    }
}
