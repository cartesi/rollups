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

import {IENSResolutionRelay} from "./IENSResolutionRelay.sol";
import {InputRelay} from "../inputs/InputRelay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

abstract contract Resolver {
    function addr(bytes32 node) public view virtual returns (address);
}

abstract contract ENS {
    function resolver(bytes32 node) public view virtual returns (Resolver);
}

/// @title ENS Resolution Relay
///
/// @notice This contract allows anyone to inform a dapp's off-chain machine
/// of an ENS node and its resolution in a trustless and permissionless way.
contract ENSResolutionRelay is InputRelay, IENSResolutionRelay {
    // Same address for Mainet, Ropsten, Rinkerby, Gorli and other networks;
    ENS ens = ENS(0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e);

    /// @notice Constructs the relay.
    /// @param _inputBox The input box used by the relay
    constructor(IInputBox _inputBox) InputRelay(_inputBox) {}

    /// @dev It is possible that the resolution is address 0.
    /// `_node` can be computed off chain.
    /// Make sure the resolution in the input box is up-to-date,
    /// by calling this function once there's any change.
    function relayENSResolution(
        address _dapp,
        bytes32 _node
    ) external override {
        Resolver resolver = ens.resolver(_node);
        address resolution = resolver.addr(_node);

        bytes memory input = InputEncoding.encodeENSResolutionRelay(
            _node,
            resolution
        );
        inputBox.addInput(_dapp, input);
    }
}
