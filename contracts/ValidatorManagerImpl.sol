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

/// @title Validator Manager Implementation
pragma solidity ^0.7.0;

import "./ValidatorManager.sol";

contract ValidatorManagerImpl is ValidatorManager {
    address immutable descartesV2; // descartes 2 contract using this validator
    bytes32 currentClaim; // current claim - first claim of this epoch
    address payable[] validators; // current validators

    // A bit set for each validator that agrees with current claim,
    // on their respective positions
    uint32 claimAgreementMask;

    // Every validator who should approve (in order to reach consensus) will have a one set on this mask
    // This mask is updated if a validator is added or removed
    uint32 consensusGoalMask;

    // @notice functions modified by onlyDescartesV2 will only be executed if
    // they're called by DescartesV2 contract, otherwise it will throw an exception
    modifier onlyDescartesV2 {
        require(
            msg.sender == descartesV2,
            "Only descartesV2 can call this functions"
        );
        _;
    }

    // @notice populates validators array and creates a consensus mask
    // @params _descartesV2 address of descartes contract
    // @params _validators initial validator set
    constructor(address _descartesV2, address payable[] memory _validators) {
        descartesV2 = _descartesV2;
        validators = _validators;

        // create consensus goal, represents the scenario where all
        // all validators claimed and agreed
        consensusGoalMask = updateConsensusGoalMask();
    }

    // @notice called when a claim is received by descartesv2
    // @params _sender address of sender of that claim
    // @params _claim claim received by descartesv2
    // @return result of claim, Consensus | NoConflict | Conflict
    // @return [currentClaim, conflicting claim] if there is Conflict
    //         [currentClaim, bytes32(0)] if there is Consensus
    //         [bytes32(0), bytes32(0)] if there is NoConflcit
    // @return [claimer1, claimer2] if there is  Conflcit
    //         [claimer1, address(0)] if there is Consensus
    //         [address(0), address(0)] if there is NoConflcit
    function onClaim(address payable _sender, bytes32 _claim)
        public
        override
        onlyDescartesV2
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        require(_claim != bytes32(0), "claim of bytes32(0) is invalid");

        // TODO: should claims by non validators just revert?
        if (!isAllowed(_sender))
            return (
                Result.NoConflict,
                [bytes32(0), bytes32(0)],
                [payable(0), payable(0)]
            );

        if (currentClaim == bytes32(0)) {
            currentClaim = _claim;
        }

        if (_claim != currentClaim) {
            return (
                Result.Conflict,
                [currentClaim, _claim],
                [getClaimerOfCurrentClaim(), _sender]
            );
        }

        return
            isConsensus(updateClaimAgreementMask(_sender), consensusGoalMask)
                ? (
                    Result.Consensus,
                    [_claim, bytes32(0)],
                    [_sender, payable(0)]
                )
                : (
                    Result.NoConflict,
                    [bytes32(0), bytes32(0)],
                    [payable(0), payable(0)]
                );
    }

    // @notice called when a dispute ends in descartesv2
    // @params _winner address of dispute winner
    // @params _loser address of dispute loser
    // @returns result of dispute being finished
    function onDisputeEnd(
        address payable _winner,
        address payable _loser,
        bytes32 _winningClaim
    )
        public
        override
        onlyDescartesV2
        returns (
            Result,
            bytes32[2] memory,
            address payable[2] memory
        )
    {
        // remove validator also removes validator from both bitmask
        (
            claimAgreementMask,
            consensusGoalMask
        ) = removeFromValidatorSetAndBothBitmasks(_loser);

        if (_winningClaim == currentClaim) {
            // first claim stood, dont need to update the bitmask
            return
                isConsensus(claimAgreementMask, consensusGoalMask)
                    ? (
                        Result.Consensus,
                        [_winningClaim, bytes32(0)],
                        [_winner, payable(0)]
                    )
                    : (
                        Result.NoConflict,
                        [bytes32(0), bytes32(0)],
                        [payable(0), payable(0)]
                    );
        }

        // if first claim lost, and other validators have agreed with it
        // there is a new dispute to be played
        if (claimAgreementMask != 0) {
            return (
                Result.Conflict,
                [currentClaim, _winningClaim],
                [getClaimerOfCurrentClaim(), _winner]
            );
        }
        // else there are no valdiators that agree with losing claim
        // but we check for consensus in case the winner is the only validator left
        return
            isConsensus(updateClaimAgreementMask(_winner), consensusGoalMask)
                ? (
                    Result.Consensus,
                    [_winningClaim, bytes32(0)],
                    [_winner, payable(0)]
                )
                : (
                    Result.NoConflict,
                    [bytes32(0), bytes32(0)],
                    [payable(0), payable(0)]
                );
    }

    // @notice called when a new epoch starts
    // @return current claim
    function onNewEpoch() public override onlyDescartesV2 returns (bytes32) {
        bytes32 tmpClaim = currentClaim;

        // clear current claim
        currentClaim = bytes32(0);
        // clear validator agreement bit mask
        claimAgreementMask = 0;

        return tmpClaim;
    }

    // INTERNAL FUNCTIONS

    // @notice get one of the validators that agreed with current claim
    // @return validator that agreed with current claim
    function getClaimerOfCurrentClaim()
        internal
        view
        returns (address payable)
    {
        require(
            claimAgreementMask != 0,
            "No validators agree with current claim"
        );

        // TODO: we are always getting the first validator
        // on the array that agrees with the current claim to enter a dispute
        // should this be random?
        for (uint256 i = 0; i < validators.length; i++) {
            if (claimAgreementMask & (1 << i) == 1) {
                return validators[i];
            }
        }
        revert("Agreeing validator not found");
    }

    // @notice updates the consensus goal mask
    // @return new consensus goal mask
    function updateConsensusGoalMask() internal view returns (uint32) {
        // consensus goal is a number where
        // all bits related to validators are turned on
        uint32 consensusMask =
            (uint32(2)**uint32(validators.length)) - uint32(1);

        // the optimistc assumption is that validators getting kicked out
        // a rare event. So we save gas by starting with the optimistic scenario
        // and turning the bits off for removed validators
        for (uint32 i = 0; i < validators.length; i++) {
            if (validators[i] == address(0)) {
                uint32 zeroMask = ~(uint32(1) << i);
                consensusMask = consensusMask & zeroMask;
            }
        }
        return consensusMask;
    }

    // @notice updates mask of validators that agreed with current claim
    // @params _sender address that of validator that will be included in mask
    // @return true if mask update led to consensus, false if not
    function updateClaimAgreementMask(address _sender)
        internal
        returns (uint32)
    {
        for (uint32 i = 0; i < validators.length; i++) {
            if (_sender == validators[i]) {
                claimAgreementMask = claimAgreementMask | (uint32(1) << i);
                break;
            }
        }

        return claimAgreementMask;
    }

    // @notice removes a validator
    // @params address of validator to be removed
    // @returns new claim agreement bitmask
    // @returns new consensus goal bitmask
    function removeFromValidatorSetAndBothBitmasks(address _validator)
        internal
        returns (uint32, uint32)
    {
        uint32 newClaimAgreementMask;
        uint32 newConsensusGoalMask;
        // put address(0) in validators position
        // removes validator from claim agreement bitmask
        // removes validator from consensus goal mask
        for (uint32 i = 0; i < validators.length; i++) {
            if (_validator == validators[i]) {
                validators[i] = address(0);
                uint32 zeroMask = ~(uint32(1) << i);
                newClaimAgreementMask = claimAgreementMask & zeroMask;
                newConsensusGoalMask = consensusGoalMask & zeroMask;
                break;
            }
        }
        return (newClaimAgreementMask, newConsensusGoalMask);
    }

    function isAllowed(address _sender) internal view returns (bool) {
        for (uint256 i = 0; i < validators.length; i++) {
            if (_sender == validators[i]) return true;
        }
        return false;
    }

    function isConsensus(uint32 _claimAgreementMask, uint32 _consensusGoalMask)
        internal
        pure
        returns (bool)
    {
        return _claimAgreementMask == _consensusGoalMask;
    }
}
