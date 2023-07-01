// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

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
