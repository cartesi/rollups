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

import {History} from "./History.sol";

/// @title History Factory interface
interface IHistoryFactory {
    // Events

    /// @notice A new history was deployed.
    /// @param historyOwner The initial history owner
    /// @param history The history
    /// @dev MUST be triggered on a successful call to `newHistory`.
    event HistoryCreated(address historyOwner, History history);

    // Permissionless functions

    /// @notice Deploy a new history.
    /// @param _historyOwner The initial history owner
    /// @return The history
    /// @dev On success, MUST emit a `HistoryCreated` event.
    function newHistory(address _historyOwner) external returns (History);

    /// @notice Deploy a new history deterministically.
    /// @param _historyOwner The initial history owner
    /// @param _salt The salt used to deterministically generate the history address
    /// @return The history
    /// @dev On success, MUST emit a `HistoryCreated` event.
    function newHistory(
        address _historyOwner,
        bytes32 _salt
    ) external returns (History);

    /// @notice Calculate the address of a history to be deployed deterministically.
    /// @param _historyOwner The initial history owner
    /// @param _salt The salt used to deterministically generate the history address
    /// @return The deterministic history address
    /// @dev Beware that only the `newHistory` function with the `_salt` parameter
    ///      is able to deterministically deploy a history.
    function calculateHistoryAddress(
        address _historyOwner,
        bytes32 _salt
    ) external view returns (address);
}
