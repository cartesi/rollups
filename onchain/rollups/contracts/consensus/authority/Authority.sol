// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Authority
pragma solidity ^0.8.13;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {AbstractConsensus} from "../AbstractConsensus.sol";
import {IInputBox} from "../../inputs/IInputBox.sol";
import {IHistory} from "../../history/IHistory.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Authority is AbstractConsensus, Ownable {
    IHistory history;

    /// @notice A consensus was created
    /// @param owner The address that owns the consensus
    /// @param inputBox The input box used by the consensus
    /// @param history The history that the consensus writes to
    event ConsensusCreated(address owner, IInputBox inputBox, IHistory history);

    /// @notice A new history is used
    /// @param history The new history
    event NewHistory(IHistory history);

    constructor(address _owner, IInputBox _inputBox, IHistory _history) {
        // constructor in Ownable already called `transferOwnership(msg.sender)`, so
        // we only need to call `transferOwnership(_owner)` if _owner != msg.sender
        if (msg.sender != _owner) {
            transferOwnership(_owner);
        }
        history = _history;
        emit ConsensusCreated(_owner, _inputBox, _history);
    }

    /// @dev Will fail if history has migrated to another consensus
    function submitClaim(bytes calldata _claimData) external onlyOwner {
        history.submitClaim(_claimData);
    }

    function migrateHistoryToConsensus(address _consensus) external onlyOwner {
        history.migrateToConsensus(_consensus);
    }

    function setHistory(IHistory _history) external onlyOwner {
        history = _history;
        emit NewHistory(_history);
    }

    function getHistory() external view returns (IHistory) {
        return history;
    }

    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        return history.getClaim(_dapp, _proofContext);
    }

    function withdrawERC20Tokens(
        IERC20 _token,
        address _recipient,
        uint256 _amount
    ) external onlyOwner {
        require(
            _token.transfer(_recipient, _amount),
            "Authority: withdrawal failed"
        );
    }
}
