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

interface MockVoucher {
    /// @notice executes voucher
    /// @param _destination address that will execute voucher
    /// @param _payload payload to be executed by destination
    /// @param _epochIndex which epoch the voucher belongs to
    /// @param _inputIndex which input, inside the epoch, the voucher belongs to
    /// @param _voucherIndex index of voucher inside the input
    /// @param _vouchersHash hash of the vouchers drive where this voucher is contained
    /// @param _voucherProof bytes that describe the voucher, can encode different things
    /// @param _epochProof siblings of vouchers hash, to prove it is contained on epoch hash
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        uint256 _epochIndex,
        uint256 _inputIndex,
        uint256 _voucherIndex,
        bytes32 _vouchersHash,
        bytes32[] calldata _voucherProof,
        bytes32[] calldata _epochProof
    ) external returns (bool);

    /// @notice called by rollups when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its vouchers can be called
    function onNewEpoch(bytes32 _epochHash) external;

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs() external view returns (uint256);
}
