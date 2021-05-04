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

/// @title Output Implementation
pragma solidity ^0.7.0;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import "./Output.sol";

contract OutputImpl is Output {
    using SafeMath for uint256;
    using Bitmask for mapping(uint248 => uint256);

    uint8 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    address immutable descartesV2; // descartes 2 contract using this validator

    mapping(uint248 => uint256) internal outputBitmask;
    bytes32[] epochHashes;

    bool lock; //reentrancy lock

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    // @notice functions modified by onlyDescartesV2 will only be executed if
    // they're called by DescartesV2 contract, otherwise it will throw an exception
    modifier onlyDescartesV2 {
        require(
            msg.sender == descartesV2,
            "Only descartesV2 can call this functions"
        );
        _;
    }

    // @notice creates OutputImpl contract
    // @params _descartesV2 address of descartes contract
    constructor(address _descartesV2) {
        descartesV2 = _descartesV2;
    }

    /// @notice executes output
    /// @param _destination address that will execute output
    /// @param _payload payload to be executed by destination
    /// @param _epochIndex which epoch the output belongs to
    /// @param _inputIndex which input, inside the epoch, the output belongs to
    /// @param _outputIndex index of output inside the input
    /// @param _outputDriveHash hash of the outputs drive where this output is contained
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
        bytes32 _outputDriveHash,
        bytes32[] calldata _outputProof,
        bytes32[] calldata _epochProof
    ) public override noReentrancy returns (bool) {
        uint256 bitmaskPosition =
            getBitMaskPosition(_outputIndex, _inputIndex, _epochIndex);

        require(
            !outputBitmask.getBit(bitmaskPosition),
            "output has already been executed"
        );

        bytes32 hashOfOutput =
            keccak256(abi.encodePacked(_destination, _payload));

        // prove that the epoch contains that outputdrive
        require(
            Merkle.getRootWithDrive(
                uint64(_outputIndex.mul(KECCAK_LOG2_SIZE)),
                KECCAK_LOG2_SIZE,
                hashOfOutput,
                _outputProof
            ) == _outputDriveHash,
            "specific output is not contained in output drive merkle hash"
        );

        // prove that epoch hash contains the claimed outputs hash
        require(
            Merkle.getRootWithDrive(
                uint64(_inputIndex.mul(KECCAK_LOG2_SIZE)),
                KECCAK_LOG2_SIZE,
                _outputDriveHash,
                _epochProof
            ) == epochHashes[_epochIndex],
            "output drive hash not contained in epochHashes"
        );

        // do we need return data? emit event?
        (bool succ, bytes memory returnData) =
            address(_destination).call(_payload);

        if (succ) outputBitmask.setBit(bitmaskPosition, true);

        return succ;
    }

    /// @notice called by descartesv2 when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its outputs can be called
    function onNewEpoch(bytes32 _epochHash) public override onlyDescartesV2 {
        epochHashes.push(_epochHash);
    }

    /// @notice get output position on bitmask
    /// @param _output of output inside the input
    /// @param _input which input, inside the epoch, the output belongs to
    /// @param _epoch which epoch the output belongs to
    /// @return position of that output on bitmask
    function getBitMaskPosition(
        uint256 _output,
        uint256 _input,
        uint256 _epoch
    ) public pure returns (uint256) {
        // output * 2 ** 128 + input * 2 ** 64 + epoch
        // this can't overflow because its impossible to have > 2**128 outputs
        return (_output << 128) + (_input << 64) + _epoch;
    }

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs()
        public
        view
        override
        returns (uint256)
    {
        return epochHashes.length;
    }
}
