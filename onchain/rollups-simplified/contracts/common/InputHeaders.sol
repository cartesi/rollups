// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Headers
pragma solidity ^0.8.13;

library InputHeaders {
    /// @notice ETH deposit
    bytes1 constant ETH_DEPOSIT = hex"00";

    /// @notice ERC-20 token deposit where `transferFrom` returns `true`
    bytes1 constant ERC20_DEPOSIT_TRUE = hex"01";

    /// @notice ERC-20 token deposit where `transferFrom` returns `false`
    bytes1 constant ERC20_DEPOSIT_FALSE = hex"02";

    /// @notice ERC-721 token deposit
    bytes1 constant ERC721_DEPOSIT = hex"03";
}
