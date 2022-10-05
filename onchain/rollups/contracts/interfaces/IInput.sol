// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input interface
pragma solidity >=0.7.0;

interface IInput {
    /// @notice Adds an input to the accumulating epoch's inbox
    /// @param _input bytes array of input
    /// @return hash of the input
    /// @dev There is a maximum size for the input data that is defined by the DApp
    function addInput(bytes calldata _input) external returns (bytes32);

    /// @notice Returns the hash of the input at the provided input index, for the current sealed epoch
    /// @param _index position of the input on inbox
    /// @return hash of the input
    function getInput(uint256 _index) external view returns (bytes32);

    /// @notice Returns the number of inputs on the current sealed epoch's inbox
    /// @return number of inputs of non active inbox
    function getNumberOfInputs() external view returns (uint256);

    /// @notice Returns the internal index of the current accumulating inbox
    /// @return index of current accumulating inbox
    function getCurrentInbox() external view returns (uint256);

    /// @notice Indicates that an input was added to the accumulating epoch's inbox
    /// @param epochNumber which epoch this input belongs to
    /// @param inputIndex index of the input just added
    /// @param sender msg.sender address
    /// @param timestamp block timestamp
    /// @param input input data
    event InputAdded(
        uint256 indexed epochNumber,
        uint256 indexed inputIndex,
        address sender,
        uint256 timestamp,
        bytes input
    );
}
