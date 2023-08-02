// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "./TournamentFactory.sol";
import "../abstracts/NonLeafTournament.sol";
import "../concretes/MiddleTournament.sol";

import "../../Machine.sol";
import "../../Tree.sol";
import "../../Time.sol";

contract MiddleTournamentFactory {
    constructor() {}

    function instantiate(
        Machine.Hash _initialHash,
        Tree.Node _contestedCommitmentOne,
        Machine.Hash _contestedFinalStateOne,
        Tree.Node _contestedCommitmentTwo,
        Machine.Hash _contestedFinalStateTwo,
        Time.Duration _allowance,
        uint256 _startCycle,
        uint64 _level,
        NonLeafTournament _parent
    ) external returns (MiddleTournament) {
        MiddleTournament _tournament = new MiddleTournament(
            _initialHash,
            _contestedCommitmentOne,
            _contestedFinalStateOne,
            _contestedCommitmentTwo,
            _contestedFinalStateTwo,
            _allowance,
            _startCycle,
            _level,
            _parent,
            TournamentFactory(msg.sender)
        );

        return _tournament;
    }
}
