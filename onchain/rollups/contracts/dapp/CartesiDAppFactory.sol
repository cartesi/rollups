// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp Factory
pragma solidity ^0.8.8;

import {ICartesiDAppFactory} from "./ICartesiDAppFactory.sol";
import {IConsensus} from "../consensus/IConsensus.sol";
import {CartesiDApp} from "./CartesiDApp.sol";

contract CartesiDAppFactory is ICartesiDAppFactory {
    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) external override returns (CartesiDApp) {
        CartesiDApp application = new CartesiDApp(
            _consensus,
            _dappOwner,
            _templateHash
        );

        emit ApplicationCreated(
            _consensus,
            _dappOwner,
            _templateHash,
            application
        );

        return application;
    }
}
