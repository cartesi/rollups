// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Portal interface
pragma solidity ^0.8.8;

import {IInputBox} from "../inputs/IInputBox.sol";

interface IPortal {
    // Permissionless functions

    /// @notice Get the input box used by this portal
    /// @return the input box
    function getInputBox() external view returns (IInputBox);
}
