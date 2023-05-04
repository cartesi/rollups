// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.8;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IConsensus} from "../IConsensus.sol";
import {AbstractConsensus} from "../AbstractConsensus.sol";
import {IInputBox} from "../../inputs/IInputBox.sol";
import {IHistory} from "../../history/IHistory.sol";

/// @title Authority consensus
/// @notice A consensus model controlled by a single address, the owner.
///         Claims are stored in an auxiliary contract called `History`.
/// @dev This contract inherits `AbstractConsensus` and OpenZeppelin's `Ownable` contract.
///      For more information on `Ownable`, please consult OpenZeppelin's official documentation.
contract Authority is AbstractConsensus, Ownable {
    /// @notice The current history contract.
    /// @dev See the `getHistory` and `setHistory` functions.
    IHistory internal history;

    /// @notice The `Authority` contract was created.
    /// @param owner The address that initially owns the `Authority` contract
    /// @param inputBox The input box from which the authority fetches new inputs
    /// @dev MUST be triggered during construction.
    event ConsensusCreated(address owner, IInputBox inputBox);

    /// @notice A new history contract is used to store claims.
    /// @param history The new history contract
    /// @dev MUST be triggered on a successful call to `setHistory`.
    event NewHistory(IHistory history);

    /// @notice Constructs an `Authority` contract.
    /// @param _owner The initial contract owner
    /// @param _inputBox The input box contract
    /// @dev Emits a `ConsensusCreated` event.
    constructor(address _owner, IInputBox _inputBox) {
        // constructor in Ownable already called `transferOwnership(msg.sender)`, so
        // we only need to call `transferOwnership(_owner)` if _owner != msg.sender
        if (msg.sender != _owner) {
            transferOwnership(_owner);
        }
        emit ConsensusCreated(_owner, _inputBox);
    }

    /// @notice Submits a claim to the current history contract.
    ///         The encoding of `_claimData` might vary depending on the
    ///         implementation of the current history contract.
    /// @param _claimData Data for submitting a claim
    /// @dev Can only be called by the `Authority` owner,
    ///      and the `Authority` contract must have ownership over
    ///      its current history contract.
    function submitClaim(bytes calldata _claimData) external onlyOwner {
        history.submitClaim(_claimData);
    }

    /// @notice Transfer ownership over the current history contract to `_consensus`.
    /// @param _consensus The new owner of the current history contract
    /// @dev Can only be called by the `Authority` owner,
    ///      and the `Authority` contract must have ownership over
    ///      its current history contract.
    function migrateHistoryToConsensus(address _consensus) external onlyOwner {
        history.migrateToConsensus(_consensus);
    }

    /// @notice Make `Authority` point to another history contract.
    /// @param _history The new history contract
    /// @dev Emits a `NewHistory` event.
    ///      Can only be called by the `Authority` owner.
    function setHistory(IHistory _history) external onlyOwner {
        history = _history;
        emit NewHistory(_history);
    }

    /// @notice Get the current history contract.
    /// @return The current history contract
    function getHistory() external view returns (IHistory) {
        return history;
    }

    /// @notice Get a claim from the current history.
    ///         The encoding of `_proofContext` might vary depending on the
    ///         implementation of the current history contract.
    /// @inheritdoc IConsensus
    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    ) external view override returns (bytes32, uint256, uint256) {
        return history.getClaim(_dapp, _proofContext);
    }

    /// @notice Transfer some amount of ERC-20 tokens to a recipient.
    /// @param _token The token contract
    /// @param _recipient The recipient address
    /// @param _amount The amount of tokens to be withdrawn
    /// @dev Can only be called by the `Authority` owner.
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
