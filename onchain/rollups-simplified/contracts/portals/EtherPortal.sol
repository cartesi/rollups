// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Ether Portal
pragma solidity ^0.8.13;

import {IEtherPortal} from "./IEtherPortal.sol";
import {InputBox} from "../inputs/InputBox.sol";
import {InputHeaders} from "../common/InputHeaders.sol";

contract EtherPortal is IEtherPortal {
    InputBox public immutable inputBox;

    constructor(InputBox _inputBox) {
        inputBox = _inputBox;
    }

    function depositEther(address _dapp, bytes calldata _data)
        external
        payable
        override
    {
        // We first add the input to avoid reentrancy attacks
        bytes memory input = abi.encodePacked(
            InputHeaders.ETH_DEPOSIT, // Header (1B)
            msg.sender, //               Ether sender (20B)
            msg.value, //                Ether amount (32B)
            _data //                     L2 data (arbitrary size)
        );
        inputBox.addInput(_dapp, input);

        // We used to call `transfer()` but it's not considered safe,
        // as it assumes gas costs are immutable (they are not).
        (bool success, ) = _dapp.call{value: msg.value}("");
        require(success, "EtherPortal: transfer failed");
    }
}
