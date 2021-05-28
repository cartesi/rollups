// Copyright (C) 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: GPL-3.0-only
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

/// @title Output
pragma solidity >=0.7.0;

interface Output {
    /// @param _epochIndex which epoch the output belongs to
    /// @param _inputIndex which input, inside the epoch, the output belongs to
    /// @param _outputIndex index of output inside the input
    /// @param _outputMetadataDriveHash hash of the outputs metadata drive where this output is in
    /// @param _outputsHash merkle root of all epoch's output metadata drive hashes
    /// @param _stateHash hash of the machine state claimed this epoch
    /// @param _eventsHash hash of the events emitted by this epoch
    /// @param _outputMetadataProof proof that this output's metadata is in meta data drive
    /// @param _accumulatedOutputsProof proof that this output metadata drive is in epoch's Output drive
    struct OutputValidityProof {
        uint256 epochIndex;
        uint256 inputIndex;
        uint256 outputIndex;
        bytes32 outputMetadataDriveHash;
        bytes32 outputsHash;
        bytes32 stateHash;
        bytes32 eventsHash;
        bytes32[] outputMetadataProof;
        bytes32[] accumulatedOutputsProof;
    }

    /// @notice executes output
    /// @param _destination address that will execute the payload
    /// @param _payload payload to be executed by destination
    /// @param _v validity proof for this encoded output
    /// @return true if output was executed successfully
    /// @dev  outputs can only be executed once
    function executeOutput(
        address _destination,
        bytes calldata _payload,
        OutputValidityProof calldata _v
    ) external returns (bool);

    /// @notice called by descartesv2 when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its outputs can be called
    function onNewEpoch(bytes32 _epochHash) external;

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);
}
