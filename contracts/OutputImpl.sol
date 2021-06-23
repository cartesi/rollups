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
pragma solidity ^0.8.0;

import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import "./Output.sol";

contract OutputImpl is Output {
    using Bitmask for mapping(uint248 => uint256);

    uint8 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    // max size of output metadata drive 32 * (2^16) bytes
    uint8 constant OUTPUT_METADATA_LOG2_SIZE = 21;
    // max size of epoch output drive 32 * (2^32) bytes
    uint8 constant EPOCH_OUTPUT_LOG2_SIZE = 37;
    uint8 immutable log2OutputMetadataArrayDriveSize;

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

    /// @notice functions modified by onlyDescartesV2 will only be executed if
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
    // @params _log2OutputMetadataArrayDriveSize log2 size
    //         of output metadata array drive
    constructor(address _descartesV2, uint8 _log2OutputMetadataArrayDriveSize) {
        descartesV2 = _descartesV2;
        log2OutputMetadataArrayDriveSize = _log2OutputMetadataArrayDriveSize;
    }

    /// @notice executes output
    /// @param _encodedOutput encoded output mocking the behaviour
    //          of abi.encode(address _destination, bytes _payload)
    /// @param _v validity proof for this encoded output
    /// @return true if output was executed successfully
    /// @dev  outputs can only be executed once
    function executeOutput(
        address _destination,
        bytes calldata _payload,
        OutputValidityProof calldata _v
    ) public override noReentrancy returns (bool) {
        bytes memory encodedOutput = abi.encode(_destination, _payload);

        // check if validity proof matches the output provided
        require(
            isValidProof(encodedOutput, epochHashes[_v.epochIndex], _v),
            "validity proof not accepted"
        );

        uint256 outputPosition =
            getBitMaskPosition(_v.outputIndex, _v.inputIndex, _v.epochIndex);

        // check if output has been executed
        require(
            !outputBitmask.getBit(outputPosition),
            "output has already been executed"
        );

        // execute output
        (bool succ, bytes memory returnData) =
            address(_destination).call(_payload);

        // if properly executed, mark it as executed
        if (succ) outputBitmask.setBit(outputPosition, true);

        return succ;
    }

    /// @notice called by descartesv2 when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its outputs can be called
    function onNewEpoch(bytes32 _epochHash) public override onlyDescartesV2 {
        epochHashes.push(_epochHash);
    }

    /// @notice functions modified by validProof will only be executed if
    //  the validity proof is valid
    function isValidProof(
        bytes memory _encodedOutput,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) public pure returns (bool) {
        // prove that outputs hash is represented in a finalized epoch
        require(
            keccak256(
                abi.encodePacked(
                    _v.epochOutputDriveHash,
                    _v.epochMessageDriveHash,
                    _v.epochMachineFinalState
                )
            ) == _epochHash,
            "epoch outputs hash is not represented in the epoch hash"
        );

        // prove that output metadata drive is contained in epoch's output drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                EPOCH_OUTPUT_LOG2_SIZE,
                keccak256(abi.encodePacked(_v.outputMetadataArrayDriveHash)),
                _v.epochOutputDriveProof
            ) == _v.epochOutputDriveHash,
            "output's metadata drive hash is not contained in epoch output drive"
        );

        bytes32 hashOfOutput = keccak256(_encodedOutput);

        // The hash of the output is converted to bytes (abi.encode) and
        // treated as data. The metadata output drive stores that data while
        // being indifferent to its contents. To prove that the received
        // output is contained in the metadata output drive we need to
        // prove that x, where:
        // x = keccak(
        //          keccak(
        //              keccak(hashOfOutput[0:7]),
        //              keccak(hashOfOutput[8:15])
        //          ),
        //          keccak(
        //              keccak(hashOfOutput[16:23]),
        //              keccak(hashOfOutput[24:31])
        //          )
        //     )
        // is contained in it. We can't simply use hashOfOutput because the
        // log2size of the leaf is three (8 bytes) not  five (32 bytes)
        bytes32 merkleRootOfHashOfOutput = Merkle.getMerkleRootFromBytes(
                abi.encodePacked(hashOfOutput),
                KECCAK_LOG2_SIZE
            );

        // prove that merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.outputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                OUTPUT_METADATA_LOG2_SIZE,
                merkleRootOfHashOfOutput,
                _v.outputMetadataProof
            ) == _v.outputMetadataArrayDriveHash,
            "specific output is not contained in output metadata drive hash"
        );

        return true;
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

    /// @notice returns the position of a intra drive on a drive
    //          with  contents with the same size
    /// @param _index index of intra drive
    /// @param _log2Size of intra drive
    function getIntraDrivePosition(uint256 _index, uint8 _log2Size)
    public
    pure
    returns (uint64) {
        return uint64(_index * (1 << _log2Size));
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

    /// @notice get log2 size of output metadata drive
    function getOutputMetadataLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return OUTPUT_METADATA_LOG2_SIZE;
    }

    /// @notice get log2 size of epoch output drive
    function getEpochOutputLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return EPOCH_OUTPUT_LOG2_SIZE;
    }

}
