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

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {IConsensus} from "../IConsensus.sol";
import {InputBox} from "../../inputs/InputBox.sol";
import {ICartesiDApp} from "../../dapp/ICartesiDApp.sol";
import {ICartesiDAppFactory} from "../../dapp/ICartesiDAppFactory.sol";
import {IHistory} from "../../history/IHistory.sol";

contract Authority is IConsensus, Ownable {
    ICartesiDAppFactory dappFactory;
    IHistory history;

    constructor(
        address _owner,
        InputBox _inputBox,
        IHistory _history,
        ICartesiDAppFactory _dappFactory
    ) {
        transferOwnership(_owner);
        history = _history;
        dappFactory = _dappFactory;
        emit ConsensusCreated(_owner, _inputBox, _history, _dappFactory);
    }

    /// @dev Will fail if history has migrated to another consensus
    function submitClaim(address _dapp, bytes calldata _data)
        external
        override
        onlyOwner
    {
        history.submitClaim(_dapp, _data);
    }

    function createDApp(address _dappOwner, bytes32 _templateHash)
        external
        override
        returns (ICartesiDApp)
    {
        return dappFactory.newApplication(_dappOwner, _templateHash, this);
    }

    function migrateHistoryToConsensus(address _consensus)
        external
        override
        onlyOwner
    {
        history.migrateToConsensus(_consensus);
    }

    function setHistory(IHistory _history) external override onlyOwner {
        history = _history;
        emit NewHistory(_history);
    }

    /// @dev The new factory implementation must have the same interface.
    ///      If the interface of the factory must change, you need to deploy
    ///      a new version of the consensus contract and migrate to it.
    function setDAppFactory(ICartesiDAppFactory _dappFactory)
        external
        override
        onlyOwner
    {
        dappFactory = _dappFactory;
        emit NewDAppFactory(_dappFactory);
    }

    function getHistory() external view override returns (IHistory) {
        return history;
    }

    function getDAppFactory()
        external
        view
        override
        returns (ICartesiDAppFactory)
    {
        return dappFactory;
    }

    function getEpochHash(address _dapp, bytes calldata _data)
        external
        view
        returns (
            bytes32,
            uint256,
            uint256
        )
    {
        return history.getEpochHash(_dapp, _data);
    }
}
