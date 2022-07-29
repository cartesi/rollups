// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Authority interface
pragma solidity ^0.8.13;

import {ICartesiDApp} from "../../dapp/ICartesiDApp.sol";

interface IAuthority {
    function submitFinalizedClaim(
        address _dapp,
        bytes32 _finalizedClaim,
        uint256 _lastFinalizedInput
    ) external;

    function createDApp(address _dappOwner, bytes32 _templateHash)
        external
        returns (ICartesiDApp);

    function changeFactoryImpl(address _cartesiDAppFactory) external;

    function migrateHistoryToConsensus(address _history, address _consensus)
        external;

    function getHistoryAddress() external view returns (address);

    event AuthorityCreated(
        address owner,
        address inputBox,
        address history,
        address cartesiDAppFactory
    );
    event DappFactoryChanged(address newFactoryAddress);
}
