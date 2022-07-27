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
import {CartesiDApp} from "../../dapp/CartesiDApp.sol";
import {ICartesiDAppFactory} from "../../dapp/ICartesiDAppFactory.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Authority is Ownable {
    event AuthorityCreated(
        address owner,
        address inputBox,
        address cartesiDAppFactory
    );
    event DappFactoryChanged(address newFactoryAddress);

    uint256 lastFinalizedInput;
    ICartesiDAppFactory cartesiDAppFactory;

    constructor(
        address _owner,
        address _inputBox,
        address _cartesiDAppFactory
    ) {
        transferOwnership(_owner);
        cartesiDAppFactory = ICartesiDAppFactory(_cartesiDAppFactory);
        emit AuthorityCreated(_owner, _inputBox, _cartesiDAppFactory);
    }

    function submitFinalizedHash(
        bytes32 _finalizedHash,
        uint256 _lastFinalizedInput,
        ICartesiDApp _dapp
    ) external onlyOwner {
        _dapp.submitFinalizedHash(_finalizedHash, _lastFinalizedInput);
    }

    // TODO: should this be payable? or only owner
    function createDApp(bytes32 _templateHash)
        public
        onlyOwner
        returns (CartesiDApp)
    {
        return cartesiDAppFactory.newApplication(_templateHash);
    }

    function changeFactoryImpl(address _cartesiDAppFactory) public onlyOwner {
        cartesiDAppFactory = ICartesiDAppFactory(_cartesiDAppFactory);
        emit DappFactoryChanged(_cartesiDAppFactory);
    }
}
