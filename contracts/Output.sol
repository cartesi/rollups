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
pragma solidity ^0.7.0;

interface Output {
    
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
}
