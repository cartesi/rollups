// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output
pragma solidity >=0.7.0;

interface MockOutput {
    /// @notice executes output
    /// @param _destination address that will execute output
    /// @param _payload payload to be executed by destination
    /// @param _epochIndex which epoch the output belongs to
    /// @param _inputIndex which input, inside the epoch, the output belongs to
    /// @param _outputIndex index of output inside the input
    /// @param _outputsHash hash of the outputs drive where this output is contained
    /// @param _outputProof bytes that describe the ouput, can encode different things
    /// @param _epochProof siblings of outputs hash, to prove it is contained on epoch hash
    /// @return true if output was executed successfully
    /// @dev  outputs can only be executed once
    function executeOutput(
        address _destination,
        bytes calldata _payload,
        uint256 _epochIndex,
        uint256 _inputIndex,
        uint256 _outputIndex,
        bytes32 _outputsHash,
        bytes32[] calldata _outputProof,
        bytes32[] calldata _epochProof
    ) external returns (bool);

    /// @notice called by descartesv2 when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its outputs can be called
    function onNewEpoch(bytes32 _epochHash) external;

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);
}
