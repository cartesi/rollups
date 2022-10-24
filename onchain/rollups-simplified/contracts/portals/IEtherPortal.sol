// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Ether Portal
pragma solidity ^0.8.13;

import {IPortal} from "./IPortal.sol";

interface IEtherPortal is IPortal {
    // Permissionless functions

    /// @notice Transfer Ether to a DApp and add an input to
    ///         the DApp's input box to signal such operation.
    /// @param _dapp The address of the DApp
    /// @param _L2data Additional data to be interpreted by L2
    /// @dev All the value sent through this function is forwarded to the DApp
    function depositEther(address _dapp, bytes calldata _L2data)
        external
        payable;
}
