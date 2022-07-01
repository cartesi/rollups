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

import {LibDS9} from "../libraries/LibDS9.sol";

contract DS9Facet {
    // Getters

    function getX() public view returns (uint32) {
        return LibDS9.diamondStorage().x;
    }

    function getY() public view returns (uint32) {
        return LibDS9.diamondStorage().y;
    }

    function getArrayLength() public view returns (uint256) {
        return LibDS9.diamondStorage().arr.length;
    }

    function getArrayElement(uint256 index) public view returns (uint256) {
        return LibDS9.diamondStorage().arr[index];
    }

    function getMappingEntry(uint256 key) public view returns (uint256) {
        return LibDS9.diamondStorage().map[key];
    }

    // Setters

    function setX(uint32 x) public {
        LibDS9.diamondStorage().x = x;
    }

    function setY(uint32 y) public {
        LibDS9.diamondStorage().y = y;
    }

    function setMappingEntry(uint256 key, uint256 value) public {
        LibDS9.diamondStorage().map[key] = value;
    }

    function addArrayElement(uint256 value) public {
        LibDS9.diamondStorage().arr.push(value);
    }
}
