// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups debug facet
pragma solidity ^0.8.0;

import {Phase} from "../interfaces/IRollups.sol";
import {LibRollups} from "../libraries/LibRollups.sol";

contract RollupsDebugFacet {
    using LibRollups for LibRollups.DiamondStorage;

    function _setInputAccumulationStart(uint32 _inputAccumulationStart) public {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        rollupsDS.inputAccumulationStart = _inputAccumulationStart;
    }

    function _setCurrentPhase(Phase _phase) public {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        rollupsDS.currentPhase_int = uint32(_phase);
    }

    function _getCurrentPhase() public view returns (Phase _phase) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        return Phase(rollupsDS.currentPhase_int);
    }

    function _getCurrentEpoch() public view returns (uint256) {
        LibRollups.DiamondStorage storage rollupsDS =
            LibRollups.diamondStorage();
        return rollupsDS.getCurrentEpoch();
    }
}
