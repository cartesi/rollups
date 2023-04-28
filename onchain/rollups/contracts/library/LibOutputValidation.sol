// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output Validation Library
pragma solidity ^0.8.8;

import {CanonicalMachine} from "../common/CanonicalMachine.sol";
import {MerkleV2} from "@cartesi/util/contracts/MerkleV2.sol";
import {OutputEncoding} from "../common/OutputEncoding.sol";

/// @param inputIndex which input, inside the epoch, the output belongs to
/// @param outputIndex index of output emitted by the input
/// @param outputHashesRootHash Merkle root of hashes of outputs emitted by the input
/// @param vouchersEpochRootHash merkle root of all epoch's voucher metadata hashes
/// @param noticesEpochRootHash merkle root of all epoch's notice metadata hashes
/// @param machineStateHash hash of the machine state claimed this epoch
/// @param keccakInHashesSiblings proof that this output metadata is in metadata memory range
/// @param outputHashesInEpochSiblings proof that this output metadata is in epoch's output memory range
struct OutputValidityProof {
    uint64 inputIndex;
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
    /// @param v the output validity proof
    /// @param encodedOutput the encoded output
    /// @param epochHash the hash of the epoch in which the output was generated
    /// @param outputsEpochRootHash either v.vouchersEpochRootHash (for vouchers)
    ///                              or v.noticesEpochRootHash (for notices)
    /// @param outputEpochLog2Size either EPOCH_VOUCHER_LOG2_SIZE (for vouchers)
    ///                             or EPOCH_NOTICE_LOG2_SIZE (for notices)
    /// @param outputHashesLog2Size either VOUCHER_METADATA_LOG2_SIZE (for vouchers)
    ///                              or NOTICE_METADATA_LOG2_SIZE (for notices)
    function validateEncodedOutput(
        OutputValidityProof calldata v,
        bytes memory encodedOutput,
        bytes32 epochHash,
        bytes32 outputsEpochRootHash,
        uint256 outputEpochLog2Size,
        uint256 outputHashesLog2Size
    ) internal pure {
        // prove that outputs hash is represented in a finalized epoch
        require(
            keccak256(
                abi.encodePacked(
                    v.vouchersEpochRootHash,
                    v.noticesEpochRootHash,
                    v.machineStateHash
                )
            ) == epochHash,
            "incorrect epochHash"
        );

        // prove that output metadata memory range is contained in epoch's output memory range
        require(
            MerkleV2.getRootAfterReplacementInDrive(
                CanonicalMachine.getIntraMemoryRangePosition(
                    v.inputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
                outputEpochLog2Size,
                v.outputHashesRootHash,
                v.outputHashesInEpochSiblings
            ) == outputsEpochRootHash,
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
        bytes32 merkleRootOfHashOfOutput = MerkleV2.getMerkleRootFromBytes(
            abi.encodePacked(keccak256(encodedOutput)),
            CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize()
        );

        // prove that Merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array memory range
        require(
            MerkleV2.getRootAfterReplacementInDrive(
                CanonicalMachine.getIntraMemoryRangePosition(
                    v.outputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
                outputHashesLog2Size,
                merkleRootOfHashOfOutput,
                v.keccakInHashesSiblings
            ) == v.outputHashesRootHash,
            "incorrect outputHashesRootHash"
        );
    }

    /// @notice Make sure the output proof is valid, otherwise revert
    /// @param v the output validity proof
    /// @param destination The contract that will execute the payload
    /// @param payload The ABI-encoded function call
    /// @param epochHash the hash of the epoch in which the output was generated
    function validateVoucher(
        OutputValidityProof calldata v,
        address destination,
        bytes calldata payload,
        bytes32 epochHash
    ) internal pure {
        bytes memory encodedVoucher = OutputEncoding.encodeVoucher(
            destination,
            payload
        );
        validateEncodedOutput(
            v,
            encodedVoucher,
            epochHash,
            v.vouchersEpochRootHash,
            CanonicalMachine.EPOCH_VOUCHER_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.VOUCHER_METADATA_LOG2_SIZE.uint64OfSize()
        );
    }

    /// @notice Make sure the output proof is valid, otherwise revert
    /// @param v the output validity proof
    /// @param notice The notice
    /// @param epochHash the hash of the epoch in which the output was generated
    function validateNotice(
        OutputValidityProof calldata v,
        bytes calldata notice,
        bytes32 epochHash
    ) internal pure {
        bytes memory encodedNotice = OutputEncoding.encodeNotice(notice);
        validateEncodedOutput(
            v,
            encodedNotice,
            epochHash,
            v.noticesEpochRootHash,
            CanonicalMachine.EPOCH_NOTICE_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.NOTICE_METADATA_LOG2_SIZE.uint64OfSize()
        );
    }

    /// @notice Get the position of a voucher on the bit mask
    /// @param voucher the index of voucher from those generated by such input
    /// @param input the index of the input in the DApp's input box
    /// @return position of the voucher on the bit mask
    function getBitMaskPosition(
        uint256 voucher,
        uint256 input
    ) internal pure returns (uint256) {
        // voucher * 2 ** 128 + input
        // this shouldn't overflow because it is impossible to have > 2**128 vouchers
        // and because we are assuming there will be < 2 ** 128 inputs on the input box
        return (((voucher << 128) | input));
    }

    /// @notice Validate input index range and get the inbox input index
    /// @param v the output validity proof
    /// @param firstInputIndex the index of the first input of the epoch in the input box
    /// @param lastInputIndex the index of the last input of the epoch in the input box
    /// @return the index of the input in the DApp's input box
    /// @dev reverts if epoch input index is not compatible with the provided input index range
    function validateInputIndexRange(
        OutputValidityProof calldata v,
        uint256 firstInputIndex,
        uint256 lastInputIndex
    ) internal pure returns (uint256) {
        uint256 inboxInputIndex = firstInputIndex + v.inputIndex;

        require(
            inboxInputIndex <= lastInputIndex,
            "inbox input index out of bounds"
        );

        return inboxInputIndex;
    }
}
