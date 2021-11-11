// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Voucher
pragma solidity >=0.7.0;

interface Voucher {
    /// @param _epochIndex which epoch the voucher belongs to
    /// @param _inputIndex which input, inside the epoch, the voucher belongs to
    /// @param _voucherIndex index of voucher inside the input
    /// @param _voucherMetadataArrayDriveHash hash of the vouchers metadata drive where this voucher is in
    /// @param _epochVoucherDriveHash merkle root of all epoch's voucher metadata drive hashes
    /// @param _epochNoticeDriveHash hash of NoticeMetadataArrayDrive
    /// @param _epochMachineFinalState hash of the machine state claimed this epoch
    /// @param _voucherMetadataProof proof that this voucher's metadata is in meta data drive
    /// @param _epochVoucherDriveProof proof that this voucher metadata drive is in epoch's Voucher drive
    struct VoucherValidityProof {
        uint256 epochIndex;
        uint256 inputIndex;
        uint256 voucherIndex;
        bytes32 voucherMetadataArrayDriveHash;
        bytes32 epochVoucherDriveHash;
        bytes32 epochNoticeDriveHash;
        bytes32 epochMachineFinalState;
        bytes32[] voucherMetadataProof;
        bytes32[] epochVoucherDriveProof;
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
        VoucherValidityProof calldata _v
    ) external returns (bool);

    /// @notice called by rollups when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its vouchers can be called
    function onNewEpoch(bytes32 _epochHash) external;

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);

    /// @notice get log2 size of voucher metadata drive
    function getVoucherMetadataLog2Size()
        external
        pure
        returns (uint256);

    /// @notice get log2 size of epoch voucher drive
    function getEpochVoucherLog2Size()
        external
        pure
        returns (uint256);

    event VoucherExecuted(uint256 voucherPosition);
}
