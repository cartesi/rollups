// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "../concretes/BottomTournament.sol";

contract BottomTournamentFactory {
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
    ) external returns (BottomTournament) {
        BottomTournament _tournament = new BottomTournament(
            _initialHash,
            _contestedCommitmentOne,
            _contestedFinalStateOne,
            _contestedCommitmentTwo,
            _contestedFinalStateTwo,
            _allowance,
            _startCycle,
            _level,
            _parent
        );

        return _tournament;
    }
}
