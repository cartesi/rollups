// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IConsensus} from "../consensus/IConsensus.sol";
import {OutputValidityProof} from "../library/LibOutputValidation.sol";

/// @notice Data for validating outputs.
/// @param validity A validity proof for the output
/// @param context Data for querying the right claim from the current consensus contract
/// @dev The encoding of `context` might vary depending on the implementation of the consensus contract.
struct Proof {
    OutputValidityProof validity;
    bytes context;
}

/// @title Cartesi DApp interface
interface ICartesiDApp {
    // Events

    /// @notice The DApp has migrated to another consensus contract.
    /// @param newConsensus The new consensus contract
    /// @dev MUST be triggered on a successful call to `migrateToConsensus`.
    event NewConsensus(IConsensus newConsensus);

    /// @notice A voucher was executed from the DApp.
    /// @param voucherId A number that uniquely identifies the voucher
    ///                  amongst all vouchers emitted by this DApp
    event VoucherExecuted(uint256 voucherId);

    // Permissioned functions

    /// @notice Migrate the DApp to a new consensus.
    /// @param _newConsensus The new consensus
    /// @dev Can only be called by the DApp owner.
    function migrateToConsensus(IConsensus _newConsensus) external;

    // Permissionless functions

    /// @notice Try to execute a voucher.
    ///
    /// Reverts if voucher was already successfully executed.
    ///
    /// @param _destination The address that will receive the payload through a message call
    /// @param _payload The payload, which—in the case of Solidity contracts—encodes a function call
    /// @param _proof The proof used to validate the voucher against
    ///               a claim submitted by the current consensus contract
    /// @return Whether the execution was successful or not
    /// @dev On a successful execution, emits a `VoucherExecuted` event.
    ///      Execution of already executed voucher will raise a `VoucherReexecutionNotAllowed` error.
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        Proof calldata _proof
    ) external returns (bool);

    /// @notice Check whether a voucher has been executed.
    /// @param _inputIndex The index of the input in the input box
    /// @param _outputIndexWithinInput The index of output emitted by the input
    /// @return Whether the voucher has been executed before
    function wasVoucherExecuted(
        uint256 _inputIndex,
        uint256 _outputIndexWithinInput
    ) external view returns (bool);

    /// @notice Validate a notice.
    /// @param _notice The notice
    /// @param _proof Data for validating outputs
    /// @return Whether the notice is valid or not
    function validateNotice(
        bytes calldata _notice,
        Proof calldata _proof
    ) external view returns (bool);

    /// @notice Get the DApp's template hash.
    /// @return The DApp's template hash
    function getTemplateHash() external view returns (bytes32);

    /// @notice Get the current consensus.
    /// @return The current consensus
    function getConsensus() external view returns (IConsensus);
}
