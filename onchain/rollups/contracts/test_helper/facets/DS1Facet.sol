// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

pragma solidity ^0.8.0;

import {LibDS1} from "../libraries/LibDS1.sol";

contract DS1Facet {
    // Getters

    function getX() public view returns (uint32) {
        return LibDS1.diamondStorage().x;
    }

    function getY() public view returns (uint32) {
        return LibDS1.diamondStorage().y;
    }

    // Setters

    function setX(uint32 x) public {
        LibDS1.diamondStorage().x = x;
    }

    function setY(uint32 y) public {
        LibDS1.diamondStorage().y = y;
    }
}
