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

import {IDAppAddressRelay} from "./IDAppAddressRelay.sol";
import {Relay} from "./Relay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title DApp Address Relay
///
/// @notice This contract allows anyone to inform the off-chain machine
/// of the address of the DApp contract in a trustless and permissionless way.
contract DAppAddressRelay is Relay, IDAppAddressRelay {
    /// @notice Constructs the relay.
    /// @param _inputBox The input box used by the relay
    constructor(IInputBox _inputBox) Relay(_inputBox) {}

    function relayDAppAddress(address _dapp) external override {
        bytes memory input = InputEncoding.encodeDAppAddressRelay(_dapp);
        inputBox.addInput(_dapp, input);
    }
}
