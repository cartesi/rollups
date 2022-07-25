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

interface IERC721Portal {
    /// @notice Deposits ERC-721 tokens into DApp's balance
    ///         and adds an input to signal such deposit.
    ///         The caller must allow this contract to withdraw
    ///         such token from their account.
    /// @param _token The address of the ERC-721 contract
    /// @param _dapp The address of the DApp
    /// @param _tokenId The identifier of the NFT being transferred
    /// @param _data Additional data to be interpreted by L2
    function depositERC721Token(
        address _token,
        address _dapp,
        uint256 _tokenId,
        bytes calldata _data
    ) external;
}
