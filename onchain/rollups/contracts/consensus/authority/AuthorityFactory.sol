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

import {Create2} from "@openzeppelin/contracts/utils/Create2.sol";

import {IAuthorityFactory} from "./IAuthorityFactory.sol";
import {Authority} from "./Authority.sol";
import {IInputBox} from "../../inputs/IInputBox.sol";

/// @title Authority Factory
/// @notice Allows anyone to reliably deploy a new `Authority` contract.
contract AuthorityFactory is IAuthorityFactory {
    function newAuthority(
        address _authorityOwner,
        IInputBox _inputBox
    ) external override returns (Authority) {
        Authority authority = new Authority(_authorityOwner, _inputBox);

        emit AuthorityCreated(_authorityOwner, _inputBox, authority);

        return authority;
    }

    function newAuthority(
        address _authorityOwner,
        IInputBox _inputBox,
        bytes32 _salt
    ) external override returns (Authority) {
        Authority authority = new Authority{salt: _salt}(
            _authorityOwner,
            _inputBox
        );

        emit AuthorityCreated(_authorityOwner, _inputBox, authority);

        return authority;
    }

    function calculateAuthorityAddress(
        address _authorityOwner,
        IInputBox _inputBox,
        bytes32 _salt
    ) external view override returns (address) {
        return
            Create2.computeAddress(
                _salt,
                keccak256(
                    abi.encodePacked(
                        type(Authority).creationCode,
                        abi.encode(_authorityOwner, _inputBox)
                    )
                )
            );
    }
}
