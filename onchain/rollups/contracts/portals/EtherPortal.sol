// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Ether Portal
pragma solidity ^0.8.8;

import {IEtherPortal} from "./IEtherPortal.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

contract EtherPortal is IEtherPortal {
    IInputBox internal immutable inputBox;

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function getInputBox() external view override returns (IInputBox) {
        return inputBox;
    }

    function depositEther(
        address _dapp,
        bytes calldata _execLayerData
    ) external payable override {
        // We used to call `transfer()` but it's not considered safe,
        // as it assumes gas costs are immutable (they are not).
        (bool success, ) = _dapp.call{value: msg.value}("");
        require(success, "EtherPortal: transfer failed");

        bytes memory input = InputEncoding.encodeEtherDeposit(
            msg.sender,
            msg.value,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
