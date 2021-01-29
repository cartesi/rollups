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


/// @title Validator Manager
pragma solidity ^0.7.0;

// TODO: What is the incentive for validators to not just copy the first claim that arrived?
interface ValidatorManager {
    // NoConflict - No conflicting claims or consensus
    // Consensus - All validators had equal claims
    // Conflict - Claim is conflicting with previous one
    enum Result {NoConflict, Consensus, Conflict};


    // @notice called when a claim is received by descartesv2
    // @params _sender address of sender of that claim
    // @params _claim claim received by descartesv2
    // @returns result of claim, signaling current state of claims
    function onClaim(
        address _sender,
        bytes32 _claim
    )
    onlyDescartesV2
    returns (Result);

    // @notice called when a dispute ends in descartesv2
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function onDisputeEnd(
        address _winner,
        address _loser
    )
    onlyDescartesV2
    returns (Result);

    // @notice called when a new epoch starts
    function onNewEpoch() onlyDescartesV2 {

    // @notice removes claim from claims[]
    // @returns claim being removed, returns 0x if there are no claims
    function popClaim() onlyDescartesV2 returns (bytes32);
}
