// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IEtherPortal} from "./IEtherPortal.sol";
import {InputRelay} from "../inputs/InputRelay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title Ether Portal
///
/// @notice This contract allows anyone to perform transfers of
/// Ether to a DApp while informing the off-chain machine.
contract EtherPortal is InputRelay, IEtherPortal {
    /// @notice Raised when the Ether transfer fails.
    error EtherTransferFailed();

    /// @notice Constructs the portal.
    /// @param _inputBox The input box used by the portal
    constructor(IInputBox _inputBox) InputRelay(_inputBox) {}

    function depositEther(
        address _dapp,
        bytes calldata _execLayerData
    ) external payable override {
        // We used to call `transfer()` but it's not considered safe,
        // as it assumes gas costs are immutable (they are not).
        (bool success, ) = _dapp.call{value: msg.value}("");

        if (!success) {
            revert EtherTransferFailed();
        }

        bytes memory input = InputEncoding.encodeEtherDeposit(
            msg.sender,
            msg.value,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
