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

import {ICartesiDAppFactory} from "./ICartesiDAppFactory.sol";
import {IConsensus} from "../consensus/IConsensus.sol";
import {CartesiDApp} from "./CartesiDApp.sol";

/// @title Cartesi DApp Factory
/// @notice Allows anyone to reliably deploy a new `CartesiDApp` contract.
contract CartesiDAppFactory is ICartesiDAppFactory {
    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) external override returns (CartesiDApp) {
        CartesiDApp application = new CartesiDApp(
            _consensus,
            _dappOwner,
            _templateHash
        );

        emit ApplicationCreated(
            _consensus,
            _dappOwner,
            _templateHash,
            application
        );

        return application;
    }

    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) external override returns (CartesiDApp) {
        CartesiDApp application = new CartesiDApp{salt: _salt}(
            _consensus,
            _dappOwner,
            _templateHash
        );

        emit ApplicationCreated(
            _consensus,
            _dappOwner,
            _templateHash,
            application
        );

        return application;
    }

    function calculateApplicationAddress(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) external view override returns (address) {
        return
            address(
                uint160(
                    uint256(
                        keccak256(
                            abi.encodePacked(
                                bytes1(0xff),
                                address(this),
                                _salt,
                                keccak256(
                                    abi.encodePacked(
                                        type(CartesiDApp).creationCode,
                                        abi.encode(
                                            _consensus,
                                            _dappOwner,
                                            _templateHash
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            );
    }
}
