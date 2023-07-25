// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {CartesiDApp} from "./CartesiDApp.sol";
import {IConsensus} from "../consensus/IConsensus.sol";

/// @title Cartesi DApp Factory interface
interface ICartesiDAppFactory {
    // Events

    /// @notice A new application was deployed.
    /// @param consensus The initial consensus contract
    /// @param dappOwner The initial DApp owner
    /// @param templateHash The initial machine state hash
    /// @param application The application
    /// @dev MUST be triggered on a successful call to `newApplication`.
    event ApplicationCreated(
        IConsensus indexed consensus,
        address dappOwner,
        bytes32 templateHash,
        CartesiDApp application
    );

    // Permissionless functions

    /// @notice Deploy a new application.
    /// @param _consensus The initial consensus contract
    /// @param _dappOwner The initial DApp owner
    /// @param _templateHash The initial machine state hash
    /// @return The application
    /// @dev On success, MUST emit an `ApplicationCreated` event.
    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash
    ) external returns (CartesiDApp);

    /// @notice Deploy a new application deterministically.
    /// @param _consensus The initial consensus contract
    /// @param _dappOwner The initial DApp owner
    /// @param _templateHash The initial machine state hash
    /// @param _salt The salt used to deterministically generate the DApp address
    /// @return The application
    /// @dev On success, MUST emit an `ApplicationCreated` event.
    function newApplication(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) external returns (CartesiDApp);

    /// @notice Calculate the address of an application to be deployed deterministically.
    /// @param _consensus The initial consensus contract
    /// @param _dappOwner The initial DApp owner
    /// @param _templateHash The initial machine state hash
    /// @param _salt The salt used to deterministically generate the DApp address
    /// @return The deterministic application address
    /// @dev Beware that only the `newApplication` function with the `_salt` parameter
    ///      is able to deterministically deploy an application.
    function calculateApplicationAddress(
        IConsensus _consensus,
        address _dappOwner,
        bytes32 _templateHash,
        bytes32 _salt
    ) external view returns (address);
}
