// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Input Box interface
pragma solidity ^0.8.13;

interface IInputBox {
    /// @notice Emitted when an input is added
    /// @param dapp The address of the DApp that received the input
    /// @param sender The address that sent the input
    /// @param input The contents of the input
    event InputAdded(address indexed dapp, address sender, bytes input);

    /// @notice Adds input to a DApp's input box
    /// @param _dapp The address of the DApp that to receive input
    /// @param _input The contents of the input
    /// @return The hash of the input sent
    function addInput(address _dapp, bytes calldata _input)
        external
        returns (bytes32);

    /// @notice Gets the number of inputs a DApp has received
    /// @param _dapp The address of the DApp
    /// @return Number of inputs in the _dapp input box
    function getNumberOfInputs(address _dapp) external view returns (uint256);
}
