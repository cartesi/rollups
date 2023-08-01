// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "../interfaces/IRootTournamentFactory.sol";
import "../concretes/SingleLevelTournament.sol";
import "../concretes/TopTournament.sol";

contract RootTournamentFactory is IRootTournamentFactory {
    IInnerTournamentFactory immutable innerFactory;

    constructor(
        IInnerTournamentFactory _innerFactory
    ) {
        innerFactory = _innerFactory;
    }

    function instantiateSingle(
        Machine.Hash _initialHash
    ) external override returns (RootTournament) {
        SingleLevelTournament _tournament = new SingleLevelTournament(
            _initialHash
        );

        emit rootCreated(_tournament);

        return _tournament;
    }

    function instantiateTopOfMultiple(
        Machine.Hash _initialHash
    ) external override returns (RootTournament) {
        TopTournament _tournament = new TopTournament(
            innerFactory,
            _initialHash
        );

        emit rootCreated(_tournament);

        return _tournament;
    }
}
