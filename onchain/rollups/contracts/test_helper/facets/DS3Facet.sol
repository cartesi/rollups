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

import {LibDS3} from "../libraries/LibDS3.sol";

contract DS3Facet {
    // Getters

    function getX() public view returns (uint32) {
        return LibDS3.diamondStorage().x;
    }

    function getY() public view returns (uint32) {
        return LibDS3.diamondStorage().y;
    }

    function getZ() public view returns (uint32) {
        return LibDS3.diamondStorage().z;
    }

    // Setters

    function setX(uint32 x) public {
        LibDS3.diamondStorage().x = x;
    }

    function setY(uint32 y) public {
        LibDS3.diamondStorage().y = y;
    }

    function setZ(uint32 z) public {
        LibDS3.diamondStorage().z = z;
    }
}
