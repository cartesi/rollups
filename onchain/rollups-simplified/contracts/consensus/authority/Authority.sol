// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Authority
pragma solidity ^0.8.13;

import {ICartesiDApp} from "../../dapp/ICartesiDApp.sol";
//import {ICartesiDAppFactory} from "../../dapp/ICartesiDAppFactory.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Authority is Ownable {
    event AuthorityCreated(address _owner, address _inputBox);
    event dappCreated();

    uint256 lastFinalizedInput;

    constructor(address _owner, address _inputBox) {
        transferOwnership(_owner);
        emit AuthorityCreated(_owner, _inputBox);
    }

    function submitFinalizedHash(
        bytes32 _finalizedHash,
        uint256 _lastFinalizedInput,
        ICartesiDApp _dapp
    ) external onlyOwner {
        _dapp.submitFinalizedHash(_finalizedHash, _lastFinalizedInput);
    }

    // TODO: should this be payable? or only owner
    function createDApp() public returns (address) {
        //       CartesiDAppFactory.
    }

    ///changeFactoryImpl((
}
