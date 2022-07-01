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

contract DS6Downgrade {
    function downgrade() external {
        LibDS6.DiamondStorage storage ds6 = LibDS6.diamondStorage();
        // we zero `x` because, if some upgrade occupies this slot,
        // it will have the default value. this could be fatal if
        // the variable that occupies the slot was an array.
        ds6.x = 0;
    }
}
