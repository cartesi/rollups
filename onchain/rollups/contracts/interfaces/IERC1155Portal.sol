// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Generic ERC1155 Portal interface
pragma solidity >=0.7.0;

import {IERC1155Receiver} from "./IERC1155Receiver.sol";

interface IERC1155Portal is IERC1155Receiver {
    /// @notice withdraw an ERC1155 token from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function erc1155Withdrawal(bytes calldata _data) external returns (bool);

    /// @notice withdraw a batch of ERC1155 tokens from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function erc1155BatchWithdrawal(
        bytes calldata _data
    ) external returns (bool);

    /// @notice emitted on a call to `onERC1155Received`
    event ERC1155Received(
        address ERC1155,
        address operator,
        address sender,
        uint256 tokenId,
        uint256 tokenAmount,
        bytes data
    );

    /// @notice emitted on a call to `onERC1155BatchReceived`
    event ERC1155BatchReceived(
        address ERC1155,
        address operator,
        address sender,
        uint256[] tokenIds,
        uint256[] tokenAmounts,
        bytes data
    );

    /// @notice emitted on ERC1155 withdrawal
    event ERC1155Withdrawn(
        address ERC1155,
        address payable receiver,
        uint256 tokenId,
        uint256 amount,
        bytes data
    );

    /// @notice emitted on ERC1155 batch withdrawal
    event ERC1155BatchWithdrawn(
        address ERC1155,
        address payable receiver,
        uint256[] tokenIds,
        uint256[] amounts,
        bytes data
    );
}
