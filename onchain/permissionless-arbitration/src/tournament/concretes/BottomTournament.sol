// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "../abstracts/LeafTournament.sol";
import "../abstracts/NonRootTournament.sol";

/// @notice Bottom tournament of a multi-level instance
contract BottomTournament is LeafTournament, NonRootTournament {
    constructor(
        Machine.Hash _initialHash,
        Tree.Node _contestedCommitmentOne,
        Machine.Hash _contestedFinalStateOne,
        Tree.Node _contestedCommitmentTwo,
        Machine.Hash _contestedFinalStateTwo,
        Time.Duration _allowance,
        uint256 _startCycle,
        uint64 _level,
        NonLeafTournament _parent
    )
        LeafTournament()
        NonRootTournament(
            _initialHash,
            _contestedCommitmentOne,
            _contestedFinalStateOne,
            _contestedCommitmentTwo,
            _contestedFinalStateTwo,
            _allowance,
            _startCycle,
            _level,
            _parent
        )
    {}
}
