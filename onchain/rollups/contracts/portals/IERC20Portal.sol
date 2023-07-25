// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/// @title ERC-20 Portal interface
interface IERC20Portal is IInputRelay {
    // Permissionless functions

    /// @notice Transfer ERC-20 tokens to a DApp and add an input to
    /// the DApp's input box to signal such operation.
    ///
    /// The caller must allow the portal to withdraw at least `_amount` tokens
    /// from their account beforehand, by calling the `approve` function in the
    /// token contract.
    ///
    /// @param _token The ERC-20 token contract
    /// @param _dapp The address of the DApp
    /// @param _amount The amount of tokens to be transferred
    /// @param _execLayerData Additional data to be interpreted by the execution layer
    function depositERC20Tokens(
        IERC20 _token,
        address _dapp,
        uint256 _amount,
        bytes calldata _execLayerData
    ) external;
}
