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
import {LibDS7} from "../libraries/LibDS7.sol";

contract DS7Downgrade {
    function downgrade() external {
        LibDS1.DiamondStorage storage ds1 = LibDS1.diamondStorage();
        LibDS7.DiamondStorage storage ds7 = LibDS7.diamondStorage();
        ds1.x = uint32(ds7.x);
        ds1.y = uint32(ds7.y);
    }
}
