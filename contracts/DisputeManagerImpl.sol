// Copyright (C) 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: GPL-3.0-only
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

/// @title DisputeManager
pragma solidity ^0.8.0;

import "./DisputeManager.sol";
import "./DescartesV2.sol";

contract DisputeManagerImpl is DisputeManager {
    DescartesV2 immutable descartesV2; // descartes 2 contract

    /// @notice functions modified by onlyDescartesV2 will only be executed if
    //  they're called by DescartesV2 contract, otherwise it will throw an exception
    modifier onlyDescartesV2 {
        require(
            msg.sender == address(descartesV2),
            "Only descartesV2 can call this functions"
        );
        _;
    }

    constructor(address _descartesV2) {
        descartesV2 = DescartesV2(_descartesV2);
    }

    /// @notice initiates a dispute betweent two players
    /// @param _claims conflicting claims
    /// @param _claimers addresses of senders of conflicting claim
    /// @dev this is a mock implementation that just gives the win
    ///      to the address in the first posititon of _claimers array
    function initiateDispute(
        bytes32[2] memory _claims,
        address payable[2] memory _claimers
    ) public override onlyDescartesV2 {
        descartesV2.resolveDispute(_claimers[0], _claimers[1], _claims[0]);
    }
}
