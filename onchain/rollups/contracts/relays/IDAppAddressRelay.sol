// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {IRelay} from "./IRelay.sol";

/// @title DApp Address Relay interface
interface IDAppAddressRelay is IRelay {
    // Permissionless functions

    /// @notice Add an input to a DApp's input box with its address.
    /// @param _dapp The address of the DApp
    function relayDAppAddress(address _dapp) external;
}
