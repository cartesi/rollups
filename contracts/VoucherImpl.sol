// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Voucher Implementation
pragma solidity ^0.8.0;

import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import "./Voucher.sol";

contract VoucherImpl is Voucher {
    // Here we only need 248 bits as keys in the mapping, but we use 256 bits for gas optimization
    using Bitmask for mapping(uint256 => uint256);

    uint256 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    // max size of voucher metadata drive 32 * (2^16) bytes
    uint256 constant VOUCHER_METADATA_LOG2_SIZE = 21;
    // max size of epoch voucher drive 32 * (2^32) bytes
    uint256 constant EPOCH_VOUCHER_LOG2_SIZE = 37;
    uint256 immutable log2VoucherMetadataArrayDriveSize;

    // max size of notice metadata drive 32 * (2^16) bytes
    uint256 constant NOTICE_METADATA_LOG2_SIZE = 21;
    // max size of epoch notice drive 32 * (2^32) bytes
    uint256 constant EPOCH_NOTICE_LOG2_SIZE = 37;
    uint256 immutable log2NoticeMetadataArrayDriveSize;

    address immutable rollups; // rollups contract using this validator
    mapping(uint256 => uint256) internal voucherBitmask;
    bytes32[] epochHashes;

    bool lock; //reentrancy lock

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    /// @notice functions modified by onlyRollups will only be executed if
    // they're called by Rollups contract, otherwise it will throw an exception
    modifier onlyRollups {
        require(msg.sender == rollups, "Only rollups");
        _;
    }

    // @notice creates VoucherImpl contract
    // @params _rollups address of rollupscontract
    // @params _log2VoucherMetadataArrayDriveSize log2 size
    //         of voucher metadata array drive
    // @params _log2NoticeMetadataArrayDriveSize log2 size
    //         of notice metadata array drive
    constructor
    (
        address _rollups,
        uint256 _log2VoucherMetadataArrayDriveSize,
        uint256 _log2NoticeMetadataArrayDriveSize
    )
    {
        rollups = _rollups;
        log2VoucherMetadataArrayDriveSize = _log2VoucherMetadataArrayDriveSize;
        log2NoticeMetadataArrayDriveSize = _log2NoticeMetadataArrayDriveSize;
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
        VoucherValidityProof calldata _v
    ) public override noReentrancy returns (bool) {
        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        // check if validity proof matches the voucher provided
        isValidVoucherProof(encodedVoucher, epochHashes[_v.epochIndex], _v);

        uint256 voucherPosition =
            getBitMaskPosition(_v.voucherIndex, _v.inputIndex, _v.epochIndex);

        // check if voucher has been executed
        require(
            !voucherBitmask.getBit(voucherPosition),
            "re-execution not allowed"
        );

        // execute voucher
        (bool succ, ) = address(_destination).call(_payload);

        // if properly executed, mark it as executed and emit event
        if (succ) {
            voucherBitmask.setBit(voucherPosition, true);
            emit VoucherExecuted(voucherPosition);
        }

        return succ;
    }

    /// @notice called by rollups when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its vouchers can be called
    function onNewEpoch(bytes32 _epochHash) public override onlyRollups {
        epochHashes.push(_epochHash);
    }

    /// @notice functions modified by validProof will only be executed if
    //  the validity proof is valid
    function isValidVoucherProof(
        bytes memory _encodedVoucher,
        bytes32 _epochHash,
        VoucherValidityProof calldata _v
    ) public pure returns (bool) {
        // prove that vouchers hash is represented in a finalized epoch
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

        // prove that voucher metadata drive is contained in epoch's voucher drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                EPOCH_VOUCHER_LOG2_SIZE,
                keccak256(abi.encodePacked(_v.voucherMetadataArrayDriveHash)),
                _v.epochVoucherDriveProof
            ) == _v.epochVoucherDriveHash,
            "epochVoucherDriveHash incorrect"
        );

        // The hash of the voucher is converted to bytes (abi.encode) and
        // treated as data. The metadata voucher drive stores that data while
        // being indifferent to its contents. To prove that the received
        // voucher is contained in the metadata voucher drive we need to
        // prove that x, where:
        // x = keccak(
        //          keccak(
        //              keccak(hashOfVoucher[0:7]),
        //              keccak(hashOfVoucher[8:15])
        //          ),
        //          keccak(
        //              keccak(hashOfVoucher[16:23]),
        //              keccak(hashOfVoucher[24:31])
        //          )
        //     )
        // is contained in it. We can't simply use hashOfVoucher because the
        // log2size of the leaf is three (8 bytes) not  five (32 bytes)
        bytes32 merkleRootOfHashOfVoucher =
            Merkle.getMerkleRootFromBytes(
                abi.encodePacked(keccak256(_encodedVoucher)),
                KECCAK_LOG2_SIZE
            );

        // prove that merkle root hash of bytes(hashOfVoucher) is contained
        // in the voucher metadata array drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.voucherIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                VOUCHER_METADATA_LOG2_SIZE,
                merkleRootOfHashOfVoucher,
                _v.voucherMetadataProof
            ) == _v.voucherMetadataArrayDriveHash,
            "voucherMetadataArrayDriveHash incorrect"
        );

        return true;
    }

    /// @notice functions modified by validProof will only be executed if
    //  the validity proof is valid
    function isValidNoticeProof(
        bytes memory _encodedNotice,
        bytes32 _epochHash,
        NoticeValidityProof calldata _v
    ) public pure returns (bool) {
        // prove that notices hash is represented in a finalized epoch
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

        // prove that notice metadata drive is contained in epoch's notice drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.inputIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                EPOCH_NOTICE_LOG2_SIZE,
                keccak256(abi.encodePacked(_v.noticeMetadataArrayDriveHash)),
                _v.epochNoticeDriveProof
            ) == _v.epochNoticeDriveHash,
            "epochNoticeDriveHash incorrect"
        );

        // The hash of the notice is converted to bytes (abi.encode) and
        // treated as data. The metadata notice drive stores that data while
        // being indifferent to its contents. To prove that the received
        // notice is contained in the metadata notice drive we need to
        // prove that x, where:
        // x = keccak(
        //          keccak(
        //              keccak(hashOfNotice[0:7]),
        //              keccak(hashOfNotice[8:15])
        //          ),
        //          keccak(
        //              keccak(hashOfNotice[16:23]),
        //              keccak(hashOfNotice[24:31])
        //          )
        //     )
        // is contained in it. We can't simply use hashOfNotice because the
        // log2size of the leaf is three (8 bytes) not  five (32 bytes)
        bytes32 merkleRootOfHashOfNotice =
            Merkle.getMerkleRootFromBytes(
                abi.encodePacked(keccak256(_encodedNotice)),
                KECCAK_LOG2_SIZE
            );

        // prove that merkle root hash of bytes(hashOfNotice) is contained
        // in the notice metadata array drive
        require(
            Merkle.getRootAfterReplacementInDrive(
                getIntraDrivePosition(_v.noticeIndex, KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                NOTICE_METADATA_LOG2_SIZE,
                merkleRootOfHashOfNotice,
                _v.noticeMetadataProof
            ) == _v.noticeMetadataArrayDriveHash,
            "noticeMetadataArrayDriveHash incorrect"
        );

        return true;
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
        return epochHashes.length;
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
}
