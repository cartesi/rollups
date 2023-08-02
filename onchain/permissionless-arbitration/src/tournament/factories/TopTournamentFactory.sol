// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "../concretes/TopTournament.sol";

contract TopTournamentFactory {
    constructor() {}

    function instantiate(
        Machine.Hash _initialHash
    ) external returns (TopTournament) {
        TopTournament _tournament = new TopTournament(
            _initialHash,
            TournamentFactory(msg.sender)
        );

        return _tournament;
    }
}
