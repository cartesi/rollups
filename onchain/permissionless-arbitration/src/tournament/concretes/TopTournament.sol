// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.17;

import "../abstracts/RootTournament.sol";
import "../abstracts/NonLeafTournament.sol";

import "../factories/TournamentFactory.sol";

import "../../Machine.sol";

/// @notice Top tournament of a multi-level instance
contract TopTournament is NonLeafTournament, RootTournament {
    constructor(
        Machine.Hash _initialHash,
        TournamentFactory _factory
    )
        NonLeafTournament(_factory)
        RootTournament(_initialHash)
    {}
}
