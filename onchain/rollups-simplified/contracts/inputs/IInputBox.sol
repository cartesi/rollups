// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Box interface
pragma solidity ^0.8.13;

interface IInputBox {
    /// @notice Emitted when an input is added to a DApp's input box
    /// @param dapp The address of the DApp
    /// @param inputIndex The index of the input
    /// @param sender The address that sent the input
    /// @param input The contents of the input
    event InputAdded(address indexed dapp, uint256 indexed inputIndex, address sender, bytes input);

    /// @notice Add an input to a DApp's input box
    /// @param _dapp The address of the DApp
    /// @param _input The contents of the input
    /// @return The hash of the input plus some extra metadata
    function addInput(address _dapp, bytes calldata _input)
        external
        returns (bytes32);

    /// @notice Get the number of inputs in a DApp's input box
    /// @param _dapp The address of the DApp
    /// @return Number of inputs in the DApp's input box
    function getNumberOfInputs(address _dapp) external view returns (uint256);

    /// @notice Get the hash of an input in a DApp's input box
    /// @param _dapp The address of the DApp
    /// @param _index The index of the input in the DApp's input box
    /// @return The hash of the input at the provided index in the DApp's input box
    function getInputHash(address _dapp, uint256 _index)
        external
        view
        returns (bytes32);
}
