// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output Validation Library
pragma solidity ^0.8.13;

import {CanonicalMachine} from "../common/CanonicalMachine.sol";
import {Merkle} from "@cartesi/util/contracts/Merkle.sol";
import {OutputEncoding} from "../common/OutputEncoding.sol";

/// @param outputIndex index of output emitted by the input
/// @param outputHashesRootHash Merkle root of hashes of outputs emitted by the input
/// @param vouchersEpochRootHash merkle root of all epoch's voucher metadata hashes
/// @param noticesEpochRootHash merkle root of all epoch's notice metadata hashes
/// @param machineStateHash hash of the machine state claimed this epoch
/// @param keccakInHashesSiblings proof that this output metadata is in metadata memory range
/// @param outputHashesInEpochSiblings proof that this output metadata is in epoch's output memory range
struct OutputValidityProof {
    uint64 outputIndex;
    bytes32 outputHashesRootHash;
    bytes32 vouchersEpochRootHash;
    bytes32 noticesEpochRootHash;
    bytes32 machineStateHash;
    bytes32[] keccakInHashesSiblings;
    bytes32[] outputHashesInEpochSiblings;
}

library LibOutputValidation {
    using CanonicalMachine for CanonicalMachine.Log2Size;

    /// @notice Make sure the output proof is valid, otherwise revert
    /// @param _v the output validity proof
    /// @param _encodedOutput the encoded output
    /// @param _epochHash the hash of the epoch in which the output was generated
    /// @param _epochInputIndex index of input in the epoch
    /// @param _outputsEpochRootHash either _v.vouchersEpochRootHash (for vouchers)
    ///                              or _v.noticesEpochRootHash (for notices)
    /// @param _outputEpochLog2Size either EPOCH_VOUCHER_LOG2_SIZE (for vouchers)
    ///                             or EPOCH_NOTICE_LOG2_SIZE (for notices)
    /// @param _outputHashesLog2Size either VOUCHER_METADATA_LOG2_SIZE (for vouchers)
    ///                              or NOTICE_METADATA_LOG2_SIZE (for notices)
    function validateEncodedOutput(
        OutputValidityProof calldata _v,
        bytes memory _encodedOutput,
        bytes32 _epochHash,
        uint64 _epochInputIndex,
        bytes32 _outputsEpochRootHash,
        uint256 _outputEpochLog2Size,
        uint256 _outputHashesLog2Size
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
            "incorrect epochHash"
        );

        // prove that output metadata memory range is contained in epoch's output memory range
        require(
            Merkle.getRootAfterReplacementInDrive(
                CanonicalMachine.getIntraMemoryRangePosition(
                    _epochInputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
                _outputEpochLog2Size,
                _v.outputHashesRootHash,
                _v.outputHashesInEpochSiblings
            ) == _outputsEpochRootHash,
            "incorrect outputsEpochRootHash"
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
            CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize()
        );

        // prove that Merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array memory range
        require(
            Merkle.getRootAfterReplacementInDrive(
                CanonicalMachine.getIntraMemoryRangePosition(
                    _v.outputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
                _outputHashesLog2Size,
                merkleRootOfHashOfOutput,
                _v.keccakInHashesSiblings
            ) == _v.outputHashesRootHash,
            "incorrect outputHashesRootHash"
        );
    }

    /// @notice Make sure the output proof is valid, otherwise revert
    /// @param _v the output validity proof
    /// @param _destination The contract that will execute the payload
    /// @param _payload The ABI-encoded function call
    /// @param _epochHash the hash of the epoch in which the output was generated
    /// @param _epochInputIndex index of input in the epoch
    function validateVoucher(
        OutputValidityProof calldata _v,
        address _destination,
        bytes calldata _payload,
        bytes32 _epochHash,
        uint64 _epochInputIndex
    ) internal pure {
        bytes memory encodedVoucher = OutputEncoding.encodeVoucher(
            _destination,
            _payload
        );
        validateEncodedOutput(
            _v,
            encodedVoucher,
            _epochHash,
            _epochInputIndex,
            _v.vouchersEpochRootHash,
            CanonicalMachine.EPOCH_VOUCHER_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.VOUCHER_METADATA_LOG2_SIZE.uint64OfSize()
        );
    }

    /// @notice Make sure the output proof is valid, otherwise revert
    /// @param _v the output validity proof
    /// @param _notice The notice
    /// @param _epochHash the hash of the epoch in which the output was generated
    /// @param _epochInputIndex index of input in the epoch
    function validateNotice(
        OutputValidityProof calldata _v,
        bytes calldata _notice,
        bytes32 _epochHash,
        uint64 _epochInputIndex
    ) internal pure {
        bytes memory encodedNotice = OutputEncoding.encodeNotice(_notice);
        validateEncodedOutput(
            _v,
            encodedNotice,
            _epochHash,
            _epochInputIndex,
            _v.noticesEpochRootHash,
            CanonicalMachine.EPOCH_NOTICE_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.NOTICE_METADATA_LOG2_SIZE.uint64OfSize()
        );
    }

    /// @notice Get the position of a voucher on the bit mask
    /// @param _voucher the index of voucher from those generated by such input
    /// @param _input the index of the input in the DApp's input box
    /// @return position of the voucher on the bit mask
    function getBitMaskPosition(uint256 _voucher, uint256 _input)
        internal
        pure
        returns (uint256)
    {
        // voucher * 2 ** 128 + input
        // this shouldn't overflow because it is impossible to have > 2**128 vouchers
        // and because we are assuming there will be < 2 ** 128 inputs on the input box
        return (((_voucher << 128) | _input));
    }
}
