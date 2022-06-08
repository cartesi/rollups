// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp
pragma solidity ^0.8.13;

import "../common/CanonicalMachine.sol";
import {Merkle} from "@cartesi/util/contracts/Merkle.sol";

contract CartesiDapp {
    address consensus;
    bytes32[] outputsHashes;
    mapping(uint256 => uint256) voucherBitmask;

    event NewOutputsHash(
        uint256 indexed index,
        uint256 lastFinalizedInput,
        bytes32 hash
    );

    event NewConsensus(address newConsensus);

    constructor(address _consensus) {
        consensus = _consensus;
    }

    /// @param epochIndex which epoch the output belongs to
    /// @param inputIndex which input, inside the epoch, the output belongs to
    /// @param outputIndex index of output inside the input
    /// @param outputHashesRootHash merkle root of all epoch's output metadata hashes
    /// @param vouchersEpochRootHash merkle root of all epoch's voucher metadata hashes
    /// @param noticesEpochRootHash merkle root of all epoch's notice metadata hashes
    /// @param machineStateHash hash of the machine state claimed this epoch
    /// @param keccakInHashesSiblings proof that this output metadata is in metadata memory range
    /// @param outputHashesInEpochSiblings proof that this output metadata is in epoch's output memory range
    struct OutputValidityProof {
        uint256 epochIndex;
        uint256 inputIndex;
        uint256 outputIndex;
        bytes32 outputHashesRootHash;
        bytes32 vouchersEpochRootHash;
        bytes32 noticesEpochRootHash;
        bytes32 machineStateHash;
        bytes32[] keccakInHashesSiblings;
        bytes32[] outputHashesInEpochSiblings;
    }

    /// TODO: extend documentation
    /// @notice enforceProofValidity reverts if the proof is invalid
    ///  @dev _outputsEpochRootHash must be _v.vouchersEpochRootHash or
    ///                                  or _v.noticesEpochRootHash
    function enforceProofValidity(
        bytes memory _encodedOutput,
        bytes32 _epochHash,
        bytes32 _outputsEpochRootHash,
        uint256 _outputEpochLog2Size,
        uint256 _outputHashesLog2Size,
        OutputValidityProof calldata _v
    ) internal pure {
        // prove that outputs hash is represented in a finalized epoch
        require(
            keccak256(
                abi.encodePacked(
                    _v.vouchersEpochRootHash,
                    _v.noticesEpochRootHash,
                    _v.machineStateHash
                )
            ) == _epochHash,
            "epochHash incorrect"
        );

        // prove that output metadata memory range is contained in epoch's output memory range
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, CanonicalMachine.KECCAK_LOG2_SIZE),
                CanonicalMachine.KECCAK_LOG2_SIZE,
                _outputEpochLog2Size,
                keccak256(abi.encodePacked(_v.outputHashesRootHash)),
                _v.outputHashesInEpochSiblings
            ) == _outputsEpochRootHash,
            "outputsEpochRootHash incorrect"
        );

        // The hash of the output is converted to bytes (abi.encode) and
        // treated as data. The metadata output memory range stores that data while
        // being indifferent to its contents. To prove that the received
        // output is contained in the metadata output memory range we need to
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
            abi.encodePacked(keccak256(_encodedOutput)),
            CanonicalMachine.KECCAK_LOG2_SIZE
        );

        // prove that merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array memory range
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.outputIndex, CanonicalMachine.KECCAK_LOG2_SIZE),
                CanonicalMachine.KECCAK_LOG2_SIZE,
                _outputHashesLog2Size,
                merkleRootOfHashOfOutput,
                _v.keccakInHashesSiblings
            ) == _v.outputHashesRootHash,
            "outputHashesRootHash incorrect"
        );
    }

    /// @notice returns the position of a intra memory range on a memory range
    //          with  contents with the same size
    /// @param _index index of intra memory range
    /// @param _log2Size of intra memory range
    function getIntraDrivePosition(uint256 _index, uint256 _log2Size)
        public
        pure
        returns (uint256)
    {
        return (_index << _log2Size);
    }
}
