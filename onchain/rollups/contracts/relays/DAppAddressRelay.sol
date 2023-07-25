// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IDAppAddressRelay} from "./IDAppAddressRelay.sol";
import {InputRelay} from "../inputs/InputRelay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title DApp Address Relay
///
/// @notice This contract allows anyone to inform the off-chain machine
/// of the address of the DApp contract in a trustless and permissionless way.
contract DAppAddressRelay is InputRelay, IDAppAddressRelay {
    /// @notice Constructs the relay.
    /// @param _inputBox The input box used by the relay
    constructor(IInputBox _inputBox) InputRelay(_inputBox) {}

    function relayDAppAddress(address _dapp) external override {
        bytes memory input = InputEncoding.encodeDAppAddressRelay(_dapp);
        inputBox.addInput(_dapp, input);
    }
}
