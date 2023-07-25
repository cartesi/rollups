// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {Create2} from "@openzeppelin/contracts/utils/Create2.sol";

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
            Create2.computeAddress(
                _salt,
                keccak256(
                    abi.encodePacked(
                        type(CartesiDApp).creationCode,
                        abi.encode(_consensus, _dappOwner, _templateHash)
                    )
                )
            );
    }
}
