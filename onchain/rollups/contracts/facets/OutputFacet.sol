// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output facet
pragma solidity ^0.8.0;

import {Bitmask} from "@cartesi/util/contracts/Bitmask.sol";
import {MerkleV2} from "@cartesi/util/contracts/MerkleV2.sol";

import {IOutput, OutputValidityProof} from "../interfaces/IOutput.sol";

import {LibOutput} from "../libraries/LibOutput.sol";
import {LibFeeManager} from "../libraries/LibFeeManager.sol";

contract OutputFacet is IOutput {
    using LibOutput for LibOutput.DiamondStorage;

    // Here we only need 248 bits as keys in the mapping, but we use 256 bits for gas optimization
    using Bitmask for mapping(uint256 => uint256);

    uint256 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    // max size of voucher metadata memory range 32 * (2^16) bytes
    uint256 constant VOUCHER_METADATA_LOG2_SIZE = 21;
    // max size of epoch voucher memory range 32 * (2^32) bytes
    uint256 constant EPOCH_VOUCHER_LOG2_SIZE = 37;

    // max size of notice metadata memory range 32 * (2^16) bytes
    uint256 constant NOTICE_METADATA_LOG2_SIZE = 21;
    // max size of epoch notice memory range 32 * (2^32) bytes
    uint256 constant EPOCH_NOTICE_LOG2_SIZE = 37;

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();

        require(!outputDS.lock, "reentrancy not allowed");
        outputDS.lock = true;
        _;
        outputDS.lock = false;
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
    ) public override noReentrancy returns (bool) {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();
        LibFeeManager.DiamondStorage storage feeManagerDS = LibFeeManager
            .diamondStorage();

        // avoid a malicious DApp developer from draining the Fee Manager's bank account
        require(_destination != address(feeManagerDS.bank), "bad destination");

        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        // check if validity proof matches the voucher provided
        isValidVoucherProof(
            encodedVoucher,
            outputDS.epochHashes[_v.epochIndex],
            _v
        );

        uint256 voucherPosition = getBitMaskPosition(
            _v.outputIndex,
            _v.inputIndex,
            _v.epochIndex
        );

        // check if voucher has been executed
        require(
            !outputDS.voucherBitmask.getBit(voucherPosition),
            "re-execution not allowed"
        );

        // execute voucher
        (bool succ, ) = _destination.call(_payload);

        // if properly executed, mark it as executed and emit event
        if (succ) {
            outputDS.voucherBitmask.setBit(voucherPosition, true);
            emit VoucherExecuted(voucherPosition);
        }

        return succ;
    }

    /// @notice validates notice
    /// @param _notice notice to be verified
    /// @param _v validity proof for this notice
    /// @return true if notice is valid
    function validateNotice(
        bytes calldata _notice,
        OutputValidityProof calldata _v
    ) public view override returns (bool) {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();

        bytes memory encodedNotice = abi.encode(_notice);

        // reverts if validity proof doesnt match
        isValidNoticeProof(
            encodedNotice,
            outputDS.epochHashes[_v.epochIndex],
            _v
        );

        return true;
    }

    /// @notice isValidProof reverts if the proof is invalid
    ///  @dev _outputsEpochRootHash must be _v.vouchersEpochRootHash or
    ///                                  or _v.noticesEpochRootHash
    function isValidProof(
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
            MerkleV2.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
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
        bytes32 merkleRootOfHashOfOutput = MerkleV2.getMerkleRootFromBytes(
            abi.encodePacked(keccak256(_encodedOutput)),
            KECCAK_LOG2_SIZE
        );

        // prove that merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array memory range
        require(
            MerkleV2.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.outputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
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
        isValidProof(
            _encodedVoucher,
            _epochHash,
            _v.vouchersEpochRootHash,
            EPOCH_VOUCHER_LOG2_SIZE,
            VOUCHER_METADATA_LOG2_SIZE,
            _v
        );
    }

    /// @notice isValidNoticeProof reverts if the proof is invalid
    function isValidNoticeProof(
        bytes memory _encodedNotice,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) public pure {
        isValidProof(
            _encodedNotice,
            _epochHash,
            _v.noticesEpochRootHash,
            EPOCH_NOTICE_LOG2_SIZE,
            NOTICE_METADATA_LOG2_SIZE,
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

    /// @notice returns the position of a intra memory range on a memory range
    //          with  contents with the same size
    /// @param _index index of intra memory range
    /// @param _log2Size of intra memory range
    function getIntraDrivePosition(
        uint256 _index,
        uint256 _log2Size
    ) public pure returns (uint256) {
        return (_index << _log2Size);
    }

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs()
        public
        view
        override
        returns (uint256)
    {
        LibOutput.DiamondStorage storage outputDS = LibOutput.diamondStorage();
        return outputDS.getNumberOfFinalizedEpochs();
    }

    /// @notice get log2 size of voucher metadata memory range
    function getVoucherMetadataLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return VOUCHER_METADATA_LOG2_SIZE;
    }

    /// @notice get log2 size of epoch voucher memory range
    function getEpochVoucherLog2Size() public pure override returns (uint256) {
        return EPOCH_VOUCHER_LOG2_SIZE;
    }

    /// @notice get log2 size of notice metadata memory range
    function getNoticeMetadataLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return NOTICE_METADATA_LOG2_SIZE;
    }

    /// @notice get log2 size of epoch notice memory range
    function getEpochNoticeLog2Size() public pure override returns (uint256) {
        return EPOCH_NOTICE_LOG2_SIZE;
    }
}
