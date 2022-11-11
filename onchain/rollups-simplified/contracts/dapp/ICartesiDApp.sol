// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ICartesi DApp
pragma solidity 0.8.13;

import {IConsensus} from "../consensus/IConsensus.sol";
import {OutputValidityProof} from "../library/LibOutputValidation.sol";

interface ICartesiDApp {
    // Events

    /// @notice A new consensus is used
    /// @param newConsensus The new consensus
    event NewConsensus(IConsensus newConsensus);

    /// @notice A voucher was executed from the DApp
    /// @param voucherId A number that uniquely identifies the voucher
    ///                  amongst all vouchers emitted by this DApp
    event VoucherExecuted(uint256 voucherId);

    // Permissioned functions

    /// @notice Migrate the DApp to a new consensus
    /// @param _newConsensus The new consensus
    /// @dev Should have access control
    function migrateToConsensus(IConsensus _newConsensus) external;

    // Permissionless functions

    /// @notice Execute a version 2 voucher
    /// @param _destination The contract that will execute the payload
    /// @param _payload The ABI-encoded function call
    /// @param _claimQuery Data for querying the right claim
    /// @param _v A validity proof for the voucher
    /// @return Whether the voucher was executed successfully or not
    /// @dev The encoding of _claimQuery might vary depending on the history implementation
    /// @dev Each voucher can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        bytes calldata _claimQuery,
        OutputValidityProof calldata _v
    ) external returns (bool);

    /// @notice Validate a version 2 notice
    /// @param _notice The notice
    /// @param _claimQuery Data for querying the right claim
    /// @param _v A validity proof for the notice
    /// @return Whether the notice is valid or not
    /// @dev The encoding of _claimQuery might vary depending on the history implementation
    function validateNotice(
        bytes calldata _notice,
        bytes calldata _claimQuery,
        OutputValidityProof calldata _v
    ) external view returns (bool);

    /// @notice Get the DApp's template hash
    /// @return The DApp's template hash
    function getTemplateHash() external view returns (bytes32);

    /// @notice Get the current consensus
    /// @return The current consensus
    function getConsensus() external view returns (IConsensus);
}
