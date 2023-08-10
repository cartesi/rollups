// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.17;

import "../concretes/SingleLevelTournament.sol";

contract SingleLevelTournamentFactory {
    constructor() {}

    function instantiate(
        Machine.Hash _initialHash
    ) external returns (SingleLevelTournament) {
        SingleLevelTournament _tournament = new SingleLevelTournament(
            _initialHash
        );

        return _tournament;
    }
}
