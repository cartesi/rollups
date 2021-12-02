// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output debug facet
pragma solidity ^0.8.0;

import {LibOutput} from "../libraries/LibOutput.sol";

contract OutputDebugFacet {
    function _pushEpochHash(bytes32 _epochHash) public returns (uint256) {
        LibOutput.DiamondStorage storage ds = LibOutput.diamondStorage();
        ds.epochHashes.push(_epochHash);
    }
}
