// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp interface
pragma solidity ^0.8.8;

import {IConsensus} from "../consensus/IConsensus.sol";
import {OutputValidityProof} from "../library/LibOutputValidation.sol";

/// @notice Data for validating outputs
/// @param validity A validity proof for the output
/// @param context Data for querying the right claim from consensus
/// @dev The encoding of context might vary depending on the history implementation
struct Proof {
    OutputValidityProof validity;
    bytes context;
}

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

    /// @notice Execute a voucher
    /// @param _destination The contract that will execute the payload
    /// @param _payload The ABI-encoded function call
    /// @param _proof Data for validating outputs
    /// @return Whether the voucher was executed successfully or not
    /// @dev Each voucher can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        Proof calldata _proof
    ) external returns (bool);

    /// @notice Check whether a voucher has been executed
    /// @param _inboxInputIndex The index of the input in the input box
    /// @param _outputIndex The index of output emitted by the input
    /// @return Whether the voucher has been executed before
    function wasVoucherExecuted(
        uint256 _inboxInputIndex,
        uint256 _outputIndex
    ) external view returns (bool);

    /// @notice Validate a notice
    /// @param _notice The notice
    /// @param _proof Data for validating outputs
    /// @return Whether the notice is valid or not
    function validateNotice(
        bytes calldata _notice,
        Proof calldata _proof
    ) external view returns (bool);

    /// @notice Get the DApp's template hash
    /// @return The DApp's template hash
    function getTemplateHash() external view returns (bytes32);

    /// @notice Get the current consensus
    /// @return The current consensus
    function getConsensus() external view returns (IConsensus);
}
