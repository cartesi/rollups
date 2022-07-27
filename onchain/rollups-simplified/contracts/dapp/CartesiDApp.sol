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

import {CanonicalMachine} from "../common/CanonicalMachine.sol";
import {IAuthority} from "../consensus/authority/IAuthority.sol";
import {IHistory} from "../history/IHistory.sol";
import {Merkle} from "@cartesi/util/contracts/Merkle.sol";
import {Bitmask} from "@cartesi/util/contracts/Bitmask.sol";

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract CartesiDApp is ReentrancyGuard, Ownable {
    using CanonicalMachine for CanonicalMachine.Log2Size;
    using Bitmask for mapping(uint256 => uint256);

    IAuthority consensus;
    IHistory history;
    bytes32 templateHash; // state hash of the cartesi machine at t0
    mapping(uint256 => uint256) voucherBitmask;

    event NewFinalizedHash(
        uint256 indexed index,
        bytes32 finalizedHash,
        uint256 lastFinalizedInput
    );

    event NewConsensus(address newConsensus);
    event VoucherExecuted(uint256 voucherPosition);

    constructor(address _consensus, bytes32 _templateHash) {
        transferOwnership(_consensus);
        consensus = IAuthority(_consensus);
        history = IHistory(consensus.getHistoryAddress());
        templateHash = _templateHash;
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
        uint64 epochIndex;
        uint64 inputIndex;
        uint64 outputIndex;
        bytes32 outputHashesRootHash;
        bytes32 vouchersEpochRootHash;
        bytes32 noticesEpochRootHash;
        bytes32 machineStateHash;
        bytes32[] keccakInHashesSiblings;
        bytes32[] outputHashesInEpochSiblings;
    }

    /// @notice executes voucher
    /// @param _destination address that will execute the payload
    /// @param _payload payload to be executed by destination
    /// @param _v validity proof for this encoded voucher
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        OutputValidityProof calldata _v
    ) public nonReentrant returns (bool) {
        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        // check if validity proof matches the voucher provided
        bytes32[] memory claimProofs;
        bytes32 claim = history.getClaim(
            address(this),
            _v.epochIndex,
            claimProofs
        );
        isValidVoucherProof(encodedVoucher, claim, _v);

        uint256 voucherPosition = getBitMaskPosition(
            _v.outputIndex,
            _v.inputIndex,
            _v.epochIndex
        );

        // check if voucher has been executed
        require(
            voucherBitmask.getBit(voucherPosition),
            "re-execution not allowed"
        );

        // execute voucher
        (bool succ, ) = _destination.call(_payload);

        // if properly executed, mark it as executed and emit event
        if (succ) {
            voucherBitmask.setBit(voucherPosition, true);
            emit VoucherExecuted(voucherPosition);
        }

        return succ;
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
                CanonicalMachine.getIntraMemoryRangePosition(
                    _v.inputIndex,
                    CanonicalMachine.KECCAK_LOG2_SIZE
                ),
                CanonicalMachine.KECCAK_LOG2_SIZE.uint64OfSize(),
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
    ) public pure {
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
    ) public pure {
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
    /// @param _input which input, inside the epoch, the voucher belongs to
    /// @param _epoch which epoch the voucher belongs to
    /// @return position of that voucher on bitmask
    function getBitMaskPosition(
        uint256 _voucher,
        uint256 _input,
        uint256 _epoch
    ) public pure returns (uint256) {
        // voucher * 2 ** 128 + input * 2 ** 64 + epoch
        // this can't overflow because its impossible to have > 2**128 vouchers
        return (((_voucher << 128) | (_input << 64)) | _epoch);
    }
}
