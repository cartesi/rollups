// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IConsensus} from "../IConsensus.sol";
import {AbstractConsensus} from "../AbstractConsensus.sol";
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

    /// @notice A new history contract is used to store claims.
    /// @param history The new history contract
    /// @dev MUST be triggered on a successful call to `setHistory`.
    event NewHistory(IHistory history);

    /// @notice Raised when a transfer of tokens from an authority to a recipient fails.
    error AuthorityWithdrawalFailed();

    /// @notice Constructs an `Authority` contract.
    /// @param _owner The initial contract owner
    constructor(address _owner) {
        // constructor in Ownable already called `transferOwnership(msg.sender)`, so
        // we only need to call `transferOwnership(_owner)` if _owner != msg.sender
        if (msg.sender != _owner) {
            transferOwnership(_owner);
        }
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
        bool success = _token.transfer(_recipient, _amount);

        if (!success) {
            revert AuthorityWithdrawalFailed();
        }
    }
}
