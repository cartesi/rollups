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

import {IEtherPortal} from "./IEtherPortal.sol";
import {Portal} from "./Portal.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title Ether Portal
///
/// @notice This contract allows anyone to perform transfers of
/// Ether to a DApp while informing the off-chain machine.
contract EtherPortal is Portal, IEtherPortal {
    /// @notice Raised when the Ether transfer fails.
    error EtherTransferFailed();

    /// @notice Constructs the portal.
    /// @param _inputBox The input box used by the portal
    constructor(IInputBox _inputBox) Portal(_inputBox) {}

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
