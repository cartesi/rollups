// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IInputRelay} from "../inputs/IInputRelay.sol";

/// @title DApp Address Relay interface
interface IDAppAddressRelay is IInputRelay {
    // Permissionless functions

    /// @notice Add an input to a DApp's input box with its address.
    /// @param _dapp The address of the DApp
    function relayDAppAddress(address _dapp) external;
}
