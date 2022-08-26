// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Library
pragma solidity ^0.8.13;

import {CanonicalMachine} from "../common/CanonicalMachine.sol";
import {Merkle} from "@cartesi/util/contracts/Merkle.sol";

/// @param inputIndex which input, in the epoch, the output belongs to
/// @param outputIndex index of output inside the input
/// @param outputHashesRootHash merkle root of all epoch's output metadata hashes
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
                CanonicalMachine.getIntraMemoryRangePosition(
                    _v.inputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
                _outputEpochLog2Size,
                _v.outputHashesRootHash,
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
            CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize()
        );

        // prove that merkle root hash of bytes(hashOfOutput) is contained
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
            "outputHashesRootHash incorrect"
        );
    }

    /// @notice isValidVoucherProof reverts if the proof is invalid
    function isValidVoucherProof(
        bytes memory _encodedVoucher,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) internal pure {
        enforceProofValidity(
            _encodedVoucher,
            _epochHash,
            _v.vouchersEpochRootHash,
            CanonicalMachine.EPOCH_VOUCHER_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.VOUCHER_METADATA_LOG2_SIZE.uint64OfSize(),
            _v
        );
    }

    /// @notice isValidNoticeProof reverts if the proof is invalid
    function isValidNoticeProof(
        bytes memory _encodedNotice,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) internal pure {
        enforceProofValidity(
            _encodedNotice,
            _epochHash,
            _v.noticesEpochRootHash,
            CanonicalMachine.EPOCH_NOTICE_LOG2_SIZE.uint64OfSize(),
            CanonicalMachine.NOTICE_METADATA_LOG2_SIZE.uint64OfSize(),
            _v
        );
    }

    /// @notice get voucher position on bitmask
    /// @param _voucher of voucher inside the input
    /// @param _input which input, inside the input box, the voucher belongs to
    /// @return position of that voucher on bitmask
    function getBitMaskPosition(uint256 _voucher, uint256 _input)
        internal
        pure
        returns (uint256)
    {
        // voucher * 2 ** 128 + input
        // this can't overflow because it is impossible to have > 2**128 vouchers
        // and because we are assuming there won't be 2 ** 128 inputs on the input box
        return (((_voucher << 128) | _input));
    }
}
