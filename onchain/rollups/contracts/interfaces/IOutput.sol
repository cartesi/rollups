// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output interface
pragma solidity >=0.7.0;

/// @param _epochIndex which epoch the output belongs to
/// @param _inputIndex which input, inside the epoch, the output belongs to
/// @param _outputIndex index of output inside the input
/// @param _outputMetadataArrayDriveHash hash of the output's metadata drive where this output is in
/// @param _epochVoucherDriveHash merkle root of all epoch's voucher metadata drive hashes
/// @param _epochNoticeDriveHash hash of NoticeMetadataArrayDrive
/// @param _epochMachineFinalState hash of the machine state claimed this epoch
/// @param _outputMetadataProof proof that this output's metadata is in meta data drive
/// @param _epochOutputDriveProof proof that this output metadata drive is in epoch's Output drive
struct OutputValidityProof {
    uint256 epochIndex;
    uint256 inputIndex;
    uint256 outputIndex;
    bytes32 outputMetadataArrayDriveHash;
    bytes32 epochVoucherDriveHash;
    bytes32 epochNoticeDriveHash;
    bytes32 epochMachineFinalState;
    bytes32[] outputMetadataProof;
    bytes32[] epochOutputDriveProof;
}

interface IOutput {
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
    ) external returns (bool);

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);

    /// @notice get log2 size of voucher metadata drive
    function getVoucherMetadataLog2Size() external pure returns (uint256);

    /// @notice get log2 size of epoch voucher drive
    function getEpochVoucherLog2Size() external pure returns (uint256);

    /// @notice get log2 size of notice metadata drive
    function getNoticeMetadataLog2Size() external pure returns (uint256);

    /// @notice get log2 size of epoch notice drive
    function getEpochNoticeLog2Size() external pure returns (uint256);

    event VoucherExecuted(uint256 voucherPosition);
}
