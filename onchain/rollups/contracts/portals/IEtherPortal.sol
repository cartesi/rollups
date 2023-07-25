// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";

/// @title Ether Portal interface
interface IEtherPortal is IInputRelay {
    // Permissionless functions

    /// @notice Transfer Ether to a DApp and add an input to
    /// the DApp's input box to signal such operation.
    ///
    /// All the value sent through this function is forwarded to the DApp.
    ///
    /// @param _dapp The address of the DApp
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    /// @dev All the value sent through this function is forwarded to the DApp.
    ///      If the transfer fails, `EtherTransferFailed` error is raised.
    function depositEther(
        address _dapp,
        bytes calldata _execLayerData
    ) external payable;
}
