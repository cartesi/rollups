// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output Implementation
pragma solidity ^0.8.0;

import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import "./MockOutput.sol";

contract MockOutputImpl is MockOutput {
    using Bitmask for mapping(uint256 => uint256);

    uint8 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    address immutable rollups; // rollups contract using this validator

    mapping(uint256 => uint256) internal outputBitmask;
    bytes32[] epochHashes;

    bool lock; //reentrancy lock

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    // @notice functions modified by onlyRollups will only be executed if
    // they're called by Rollups contract, otherwise it will throw an exception
    modifier onlyRollups {
        require(msg.sender == rollups, "Only rollups can call this functions");
        _;
    }

    // @notice creates OutputImpl contract
    // @params _rollups address of rollupscontract
    constructor(address _rollups) {
        rollups = _rollups;
    }

    /// @notice executes output
    /// @param _destination address that will execute output
    /// @param _payload payload to be executed by destination
    /// @param _epochIndex which epoch the output belongs to
    /// @param _inputIndex which input, inside the epoch, the output belongs to
    /// @param _outputIndex index of output inside the input
    /// @param _outputDriveHash hash of the outputs drive where this output is contained
    /// @param _outputProof bytes that describe the output, can encode different things
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
            Merkle.getRootAfterReplacementInDrive(
                uint64(_outputIndex * KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                64,
                hashOfOutput,
                _outputProof
            ) == _outputDriveHash,
            "specific output is not contained in output drive merkle hash"
        );

        // prove that epoch hash contains the claimed outputs hash
        require(
            Merkle.getRootAfterReplacementInDrive(
                uint64(_inputIndex * KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                64,
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

    /// @notice called by rollups when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its outputs can be called
    function onNewEpoch(bytes32 _epochHash) public override onlyRollups {
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
