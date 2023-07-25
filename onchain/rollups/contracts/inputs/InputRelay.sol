// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "./IInputRelay.sol";
import {IInputBox} from "./IInputBox.sol";

/// @title Input Relay
/// @notice This contract serves as a base for all the other input relays.
contract InputRelay is IInputRelay {
    /// @notice The input box used by the input relay.
    IInputBox internal immutable inputBox;

    /// @notice Constructs the input relay.
    /// @param _inputBox The input box used by the input relay
    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function getInputBox() external view override returns (IInputBox) {
        return inputBox;
    }
}
