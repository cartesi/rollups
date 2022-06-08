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

contract Authority {
    event AuthorityCreated(address owner, address inputBox);
    event dappCreated();

    uint256 lastFinalizedInput;
    address owner;

    constructor(address _owner, address _inputBox) {
        owner = _owner;
        emit AuthorityCreated(_owner, _inputBox);
    }

    // TODO: onlyOwned?
    function submitClaim(bytes32 _claim, ICartesiDApp _dapp) public {
      ICartesiDApp(_dapp).submitClaim(_claim);
    }

    // TODO: should this be payable? or only owner
    function createDApp() public returns (address) {}
}
