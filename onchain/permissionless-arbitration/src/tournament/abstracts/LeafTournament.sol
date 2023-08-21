// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.17;

import "./Tournament.sol";
import "../../Commitment.sol";

import "step/ready_src/UArchStep.sol";

/// @notice Leaf tournament is the one that seals leaf match
abstract contract LeafTournament is Tournament {
    using Machine for Machine.Hash;
    using Commitment for Tree.Node;
    using Tree for Tree.Node;
    using Clock for Clock.State;
    using Match for Match.Id;
    using Match for Match.State;

    constructor() {}

    function sealLeafMatch(
        Match.Id calldata _matchId,
        Tree.Node _leftLeaf,
        Tree.Node _rightLeaf,
        Machine.Hash _agreeHash,
        bytes32[] calldata _agreeHashProof
    ) external tournamentNotFinished {
        Match.State storage _matchState = matches[_matchId.hashFromId()];
        _matchState.requireExist();
        _matchState.requireCanBeFinalized();
        _matchState.requireParentHasChildren(_leftLeaf, _rightLeaf);

        // Unpause clocks
        {
            Clock.State storage _clock1 = clocks[_matchId.commitmentOne];
            Clock.State storage _clock2 = clocks[_matchId.commitmentTwo];
            _clock1.setPaused();
            _clock1.advanceClock();
            _clock2.setPaused();
            _clock2.advanceClock();
        }

        _matchState.sealMatch(
            _matchId,
            initialHash,
            _leftLeaf,
            _rightLeaf,
            _agreeHash,
            _agreeHashProof
        );
    }

    function winLeafMatch(
        Match.Id calldata _matchId,
        Tree.Node _leftNode,
        Tree.Node _rightNode,
        bytes calldata proofs
    ) external tournamentNotFinished {
        Match.State storage _matchState = matches[_matchId.hashFromId()];
        _matchState.requireExist();
        _matchState.requireIsFinished();

        Clock.State storage _clockOne = clocks[_matchId.commitmentOne];
        Clock.State storage _clockTwo = clocks[_matchId.commitmentTwo];
        _clockOne.requireInitialized();
        _clockTwo.requireInitialized();

        (
            Machine.Hash _agreeHash,
            uint256 _agreeCycle,
            Machine.Hash _finalStateOne,
            Machine.Hash _finalStateTwo
        ) = _matchState.getDivergence(startCycle);

        Machine.Hash _finalState = runMetaStep(
            _agreeHash,
            _agreeCycle,
            proofs
        );

        if (_leftNode.join(_rightNode).eq(_matchId.commitmentOne)) {
            require(
                _finalState.eq(_finalStateOne),
                "final state one doesn't match"
            );

            _clockOne.addValidatorEffort(Time.ZERO_DURATION);
            pairCommitment(
                _matchId.commitmentOne,
                _clockOne,
                _leftNode,
                _rightNode
            );
        } else if (_leftNode.join(_rightNode).eq(_matchId.commitmentTwo)) {
            require(
                _finalState.eq(_finalStateTwo),
                "final state two doesn't match"
            );

            _clockTwo.addValidatorEffort(Time.ZERO_DURATION);
            pairCommitment(
                _matchId.commitmentTwo,
                _clockTwo,
                _leftNode,
                _rightNode
            );
        } else {
            revert("wrong left/right nodes for step");
        }

        delete matches[_matchId.hashFromId()];
    }

    function runMetaStep(Machine.Hash machineState, uint256 counter, bytes memory proofs)
         internal
         pure
         returns (Machine.Hash)
    {
        return Machine.Hash.wrap(metaStep(
            Machine.Hash.unwrap(machineState),
            counter,
            proofs
        ));
    }

    // TODO: move to step repo
    // TODO: add ureset
    function metaStep(bytes32 machineState, uint256 counter, bytes memory proofs)
         internal
         pure
         returns (bytes32)
    {
        // TODO: create a more convinient constructor.
        AccessLogs.Context memory accessLogs = AccessLogs.Context(
            machineState,
            Buffer.Context(proofs, 0)
        );

         uint256 mask = (1 << 64) - 1;
         if (counter & mask == mask) {
             // reset
             revert("RESET UNIMPLEMENTED");
         } else {
             UArchStep.step(accessLogs);
             bytes32 newMachineState = accessLogs.currentRootHash;
             return newMachineState;
         }
    }
}
