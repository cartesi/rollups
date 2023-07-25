// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputBox} from "./IInputBox.sol";

/// @title Input Relay interface
interface IInputRelay {
    // Permissionless functions

    /// @notice Get the input box used by this input relay.
    /// @return The input box
    function getInputBox() external view returns (IInputBox);
}
