// Copyright 2022 Cartesi Pte. Ltd.

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

    /// @notice validate notice
    /// @param _notice notice to be verified
    /// @param _v validity proof for this notice
    /// @return true if notice is valid
    function validateNotice(
        bytes calldata _notice,
        OutputValidityProof calldata _v
    ) external view returns (bool);

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);

    /// @notice get log2 size of voucher metadata memory range
    function getVoucherMetadataLog2Size() external pure returns (uint256);

    /// @notice get log2 size of epoch voucher memory range
    function getEpochVoucherLog2Size() external pure returns (uint256);

    /// @notice get log2 size of notice metadata memory range
    function getNoticeMetadataLog2Size() external pure returns (uint256);

    /// @notice get log2 size of epoch notice memory range
    function getEpochNoticeLog2Size() external pure returns (uint256);

    event VoucherExecuted(uint256 voucherPosition);
}
