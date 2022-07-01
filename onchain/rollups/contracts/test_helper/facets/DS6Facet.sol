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

import {LibDS6} from "../libraries/LibDS6.sol";

contract DS6Facet {
    // Getters

    function getMappingEntry(uint256 key) public view returns (uint256) {
        return LibDS6.diamondStorage().map[key];
    }

    function getArrayLength() public view returns (uint256) {
        return LibDS6.diamondStorage().arr.length;
    }

    function getArrayElement(uint256 index) public view returns (uint256) {
        return LibDS6.diamondStorage().arr[index];
    }

    function getX() public view returns (uint256) {
        return LibDS6.diamondStorage().x;
    }

    // Setters

    function setMappingEntry(uint256 key, uint256 value) public {
        LibDS6.diamondStorage().map[key] = value;
    }

    function addArrayElement(uint256 value) public {
        LibDS6.diamondStorage().arr.push(value);
    }

    function setX(uint256 x) public {
        LibDS6.diamondStorage().x = x;
    }
}
