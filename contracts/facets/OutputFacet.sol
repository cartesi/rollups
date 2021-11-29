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

import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import {IOutput, OutputValidityProof} from "../interfaces/IOutput.sol";

import {LibOutput} from "../libraries/LibOutput.sol";

contract OutputFacet is IOutput {
    // Here we only need 248 bits as keys in the mapping, but we use 256 bits for gas optimization
    using Bitmask for mapping(uint256 => uint256);

    uint256 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    // max size of voucher metadata drive 32 * (2^16) bytes
    uint256 constant VOUCHER_METADATA_LOG2_SIZE = 21;
    // max size of epoch voucher drive 32 * (2^32) bytes
    uint256 constant EPOCH_VOUCHER_LOG2_SIZE = 37;

    // max size of notice metadata drive 32 * (2^16) bytes
    uint256 constant NOTICE_METADATA_LOG2_SIZE = 21;
    // max size of epoch notice drive 32 * (2^32) bytes
    uint256 constant EPOCH_NOTICE_LOG2_SIZE = 37;

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        LibOutput.DiamondStorage storage ds = LibOutput.diamondStorage();

        require(!ds.lock, "reentrancy not allowed");
        ds.lock = true;
        _;
        ds.lock = false;
    }

    /// @notice executes voucher
    /// @param _encodedVoucher encoded voucher mocking the behaviour
    //          of abi.encode(address _destination, bytes _payload)
    /// @param _v validity proof for this encoded voucher
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        OutputValidityProof calldata _v
    ) public override noReentrancy returns (bool) {
        LibOutput.DiamondStorage storage ds = LibOutput.diamondStorage();

        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        // check if validity proof matches the voucher provided
        isValidVoucherProof(encodedVoucher, ds.epochHashes[_v.epochIndex], _v);

        uint256 voucherPosition =
            getBitMaskPosition(_v.outputIndex, _v.inputIndex, _v.epochIndex);

        // check if voucher has been executed
        require(
            !ds.voucherBitmask.getBit(voucherPosition),
            "re-execution not allowed"
        );

        // execute voucher
        (bool succ, ) = address(_destination).call(_payload);

        // if properly executed, mark it as executed and emit event
        if (succ) {
            ds.voucherBitmask.setBit(voucherPosition, true);
            emit VoucherExecuted(voucherPosition);
        }

        return succ;
    }

    /// @notice functions modified by isValidProof will only be executed if
    //  the validity proof is valid
    //  @dev _epochOutputDriveHash must be _v.epochVoucherDriveHash or
    //                                  or _v.epochNoticeDriveHash
    function isValidProof(
        bytes memory _encodedOutput,
        bytes32 _epochHash,
        bytes32 _epochOutputDriveHash,
        uint256 _epochOutputLog2Size,
        uint256 _outputMetadataLog2Size,
        OutputValidityProof calldata _v
    ) internal pure returns (bool) {
        // prove that outputs hash is represented in a finalized epoch
        require(
            keccak256(
                abi.encodePacked(
                    _v.epochVoucherDriveHash,
                    _v.epochNoticeDriveHash,
                    _v.epochMachineFinalState
                )
            ) == _epochHash,
            "epochHash incorrect"
        );

        // prove that output metadata drive is contained in epoch's output drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                _epochOutputLog2Size,
                keccak256(abi.encodePacked(_v.outputMetadataArrayDriveHash)),
                _v.epochOutputDriveProof
            ) == _epochOutputDriveHash,
            "epochOutputDriveHash incorrect"
        );

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
        bytes32 merkleRootOfHashOfOutput =
            Merkle.getMerkleRootFromBytes(
                abi.encodePacked(keccak256(_encodedOutput)),
                KECCAK_LOG2_SIZE
            );

        // prove that merkle root hash of bytes(hashOfOutput) is contained
        // in the output metadata array drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.outputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                _outputMetadataLog2Size,
                merkleRootOfHashOfOutput,
                _v.outputMetadataProof
            ) == _v.outputMetadataArrayDriveHash,
            "outputMetadataArrayDriveHash incorrect"
        );

        return true;
    }

    /// @notice functions modified by isValidVoucherProof will only be executed if
    //  the validity proof is valid
    function isValidVoucherProof(
        bytes memory _encodedVoucher,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) public pure returns (bool) {
        return
            isValidProof(
                _encodedVoucher,
                _epochHash,
                _v.epochVoucherDriveHash,
                EPOCH_VOUCHER_LOG2_SIZE,
                VOUCHER_METADATA_LOG2_SIZE,
                _v
            );
    }

    /// @notice functions modified by isValidNoticeProof will only be executed if
    //  the validity proof is valid
    function isValidNoticeProof(
        bytes memory _encodedNotice,
        bytes32 _epochHash,
        OutputValidityProof calldata _v
    ) public pure returns (bool) {
        return
            isValidProof(
                _encodedNotice,
                _epochHash,
                _v.epochNoticeDriveHash,
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

    /// @notice returns the position of a intra drive on a drive
    //          with  contents with the same size
    /// @param _index index of intra drive
    /// @param _log2Size of intra drive
    function getIntraDrivePosition(uint256 _index, uint256 _log2Size)
        public
        pure
        returns (uint256)
    {
        return (_index << _log2Size);
    }

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs()
        public
        view
        override
        returns (uint256)
    {
        return LibOutput.getNumberOfFinalizedEpochs();
    }

    /// @notice get log2 size of voucher metadata drive
    function getVoucherMetadataLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return VOUCHER_METADATA_LOG2_SIZE;
    }

    /// @notice get log2 size of epoch voucher drive
    function getEpochVoucherLog2Size() public pure override returns (uint256) {
        return EPOCH_VOUCHER_LOG2_SIZE;
    }

    /// @notice get log2 size of notice metadata drive
    function getNoticeMetadataLog2Size()
        public
        pure
        override
        returns (uint256)
    {
        return NOTICE_METADATA_LOG2_SIZE;
    }

    /// @notice get log2 size of epoch notice drive
    function getEpochNoticeLog2Size() public pure override returns (uint256) {
        return EPOCH_NOTICE_LOG2_SIZE;
    }
}
