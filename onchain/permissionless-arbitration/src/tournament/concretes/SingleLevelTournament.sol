// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "../abstracts/RootTournament.sol";
import "../abstracts/LeafTournament.sol";

contract SingleLevelTournament is LeafTournament, RootTournament {
    constructor(
        Machine.Hash _initialHash
    )
        LeafTournament()
        RootTournament(_initialHash)
    {}
}
