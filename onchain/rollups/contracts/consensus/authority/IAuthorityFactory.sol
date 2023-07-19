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

import {Authority} from "./Authority.sol";

/// @title Authority Factory interface
interface IAuthorityFactory {
    // Events

    /// @notice A new authority was deployed.
    /// @param authorityOwner The initial authority owner
    /// @param authority The authority
    /// @dev MUST be triggered on a successful call to `newAuthority`.
    event AuthorityCreated(address authorityOwner, Authority authority);

    // Permissionless functions

    /// @notice Deploy a new authority.
    /// @param _authorityOwner The initial authority owner
    /// @return The authority
    /// @dev On success, MUST emit an `AuthorityCreated` event.
    function newAuthority(address _authorityOwner) external returns (Authority);

    /// @notice Deploy a new authority deterministically.
    /// @param _authorityOwner The initial authority owner
    /// @param _salt The salt used to deterministically generate the authority address
    /// @return The authority
    /// @dev On success, MUST emit an `AuthorityCreated` event.
    function newAuthority(
        address _authorityOwner,
        bytes32 _salt
    ) external returns (Authority);

    /// @notice Calculate the address of an authority to be deployed deterministically.
    /// @param _authorityOwner The initial authority owner
    /// @param _salt The salt used to deterministically generate the authority address
    /// @return The deterministic authority address
    /// @dev Beware that only the `newAuthority` function with the `_salt` parameter
    ///      is able to deterministically deploy an authority.
    function calculateAuthorityAddress(
        address _authorityOwner,
        bytes32 _salt
    ) external view returns (address);
}
