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

import {Create2} from "@openzeppelin/contracts/utils/Create2.sol";

import {IHistoryFactory} from "./IHistoryFactory.sol";
import {History} from "./History.sol";

/// @title History Factory
/// @notice Allows anyone to reliably deploy a new `History` contract.
contract HistoryFactory is IHistoryFactory {
    function newHistory(
        address _historyOwner
    ) external override returns (History) {
        History history = new History(_historyOwner);

        emit HistoryCreated(_historyOwner, history);

        return history;
    }

    function newHistory(
        address _historyOwner,
        bytes32 _salt
    ) external override returns (History) {
        History history = new History{salt: _salt}(_historyOwner);

        emit HistoryCreated(_historyOwner, history);

        return history;
    }

    function calculateHistoryAddress(
        address _historyOwner,
        bytes32 _salt
    ) external view override returns (address) {
        return
            Create2.computeAddress(
                _salt,
                keccak256(
                    abi.encodePacked(
                        type(History).creationCode,
                        abi.encode(_historyOwner)
                    )
                )
            );
    }
}
