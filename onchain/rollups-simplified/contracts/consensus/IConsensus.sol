// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Consensus interface
pragma solidity ^0.8.13;

import {IInputBox} from "../inputs/IInputBox.sol";
import {IHistory} from "../history/IHistory.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IConsensus {
    // Events

    /// @notice A consensus was created
    /// @param owner The address that owns the consensus
    /// @param inputBox The input box used by the consensus
    /// @param history The history that the consensus writes to
    event ConsensusCreated(address owner, IInputBox inputBox, IHistory history);

    /// @notice A new history is used
    /// @param history The new history
    event NewHistory(IHistory history);

    // Permissioned functions

    /// @notice Submit a claim to history
    /// @param _claimData Data for submitting a claim
    /// @dev The encoding of _claimData might vary depending on the history implementation
    /// @dev Should have access control
    function submitClaim(bytes calldata _claimData) external;

    /// @notice Point the consensus to a new history
    /// @param _history The new history
    /// @dev Should have access control
    function setHistory(IHistory _history) external;

    /// @notice Migrate the current history to a new consensus
    /// @param _consensus The new consensus
    /// @dev Should have access control
    function migrateHistoryToConsensus(address _consensus) external;

    // Permissionless functions

    /// @notice Get the current history
    /// @return The current history
    function getHistory() external view returns (IHistory);

    /// @notice Get the epoch hash for a given DApp from a claim
    /// @param _dapp The DApp
    /// @param _claimQuery Data for querying the desired claim
    /// @return epochHash_ The epoch hash contained in the claim
    /// @return inputIndex_ The index of the input in the input box
    /// @return epochInputIndex_ The offset between the input in the input box
    //                           and the first input of the epoch in the input box
    /// @dev The encoding of _claimQuery might vary depending on the history implementation
    function getEpochHash(address _dapp, bytes calldata _claimQuery)
        external
        view
        returns (
            bytes32 epochHash_,
            uint256 inputIndex_,
            uint256 epochInputIndex_
        );

    /// @notice Transfer ERC-20 tokens from consensus contract to a recipient
    /// @param _token The ERC-20 token
    /// @param _recipient The recipient address
    /// @param _amount The transfer amount
    function withdrawERC20Tokens(
        IERC20 _token,
        address _recipient,
        uint256 _amount
    ) external;
}
