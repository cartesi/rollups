// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Factory interface
pragma solidity ^0.8.13;

import {CartesiDApp} from "./CartesiDApp.sol";
import {IConsensus} from "../consensus/IConsensus.sol";

interface ICartesiDAppFactory {
    // Events

    /// @notice A new application was deployed
    /// @param consensus The consensus to which the DApp is subscribed
    /// @param dappOwner The address that owns the DApp
    /// @param templateHash The hash of the initial state of the Cartesi Machine
    /// @param application The application
    event ApplicationCreated(
        IConsensus indexed consensus,
        address dappOwner,
        bytes32 templateHash,
        CartesiDApp application
    );

    // Permissionless functions

    /// @notice Deploy a new application
    /// @param _consensus The consensus to which the DApp should be subscribed
    /// @param _dappOwner The address that should own the DApp
    /// @param _templateHash The hash of the initial state of the Cartesi Machine
    /// @return The application
    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) external returns (CartesiDApp);
}
