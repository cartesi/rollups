// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/epoch-hash-split/EpochHashSplit.sol";
import "../../src/partition/Partition.sol";
import {Merkle} from "utils/Merkle.sol";

contract TestEpochHashSplit is Test {
    function setUp() public {}

    function test_createSplit() public {
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        Partition.Divergence memory divergence_ = createDivergence();
        divergence_.beforeHash = bytes32(
            0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
        );

        EpochHashSplit.WaitingSubhashes
            memory waitingSubhashes_ = EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );

        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceMachine),
            Merkle.Hash.unwrap(preAdvanceMachine_)
        );
        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceOutputs),
            Merkle.Hash.unwrap(preAdvanceOutputs_)
        );
        assertEq(
            waitingSubhashes_.postAdvanceEpochHashClaim,
            divergence_.afterHash
        );
        assertEq(waitingSubhashes_.inputIndex, divergence_.divergencePoint);
    }

    function testFail_createSplit() public {
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        Partition.Divergence memory divergence_ = createDivergence();

        EpochHashSplit.WaitingSubhashes
            memory waitingSubhashes_ = EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );

        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceMachine),
            Merkle.Hash.unwrap(preAdvanceMachine_)
        );
        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceOutputs),
            Merkle.Hash.unwrap(preAdvanceOutputs_)
        );
        assertEq(
            waitingSubhashes_.postAdvanceEpochHashClaim,
            divergence_.afterHash
        );
        assertEq(waitingSubhashes_.inputIndex, divergence_.divergencePoint);
    }

    function test_supplySubhashes() public {
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash postAdvanceMachineClaim_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash postAdvanceOutputsClaim_ = Merkle.Hash.wrap(INITIAL_HASH);
        Partition.Divergence memory divergence_ = createDivergence();
        /* modify the afterHash to match the expected value */
        divergence_.beforeHash = bytes32(
            0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
        );
        divergence_.afterHash = bytes32(
            0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
        );

        EpochHashSplit.WaitingSubhashes
            memory waitingSubhashes_ = EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );

        EpochHashSplit.WaitingDivergence
            memory waitingDivergence_ = EpochHashSplit.supplySubhashes(
                waitingSubhashes_,
                postAdvanceMachineClaim_,
                postAdvanceOutputsClaim_
            );

        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceMachine),
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceMachine)
        );
        assertEq(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceOutputs),
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceOutputs)
        );
        assertEq(
            Merkle.Hash.unwrap(postAdvanceMachineClaim_),
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceMachineClaim)
        );
        assertEq(
            Merkle.Hash.unwrap(postAdvanceOutputsClaim_),
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceOutputsClaim)
        );
        assertEq(waitingSubhashes_.inputIndex, waitingDivergence_.inputIndex);
    }

    function test_machineDisagree() public {
        EpochHashSplit.WaitingDivergence
            memory waitingDivergence_ = createWaitingDivergence();

        EpochHashSplit.MachineDisagree memory machineDisagree_ = EpochHashSplit
            .machineDisagree(waitingDivergence_);

        assertEq(
            Merkle.Hash.unwrap(machineDisagree_.preAdvanceMachine),
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceMachine)
        );
        assertEq(
            Merkle.Hash.unwrap(machineDisagree_.postAdvanceMachineClaim),
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceMachineClaim)
        );
        assertEq(machineDisagree_.inputIndex, waitingDivergence_.inputIndex);
    }

    function test_outputsDisagree() public {
        EpochHashSplit.WaitingDivergence
            memory waitingDivergence_ = createWaitingDivergence();

        EpochHashSplit.OutputsDisagree memory outputsDisagree_ = EpochHashSplit
            .outputsDisagree(waitingDivergence_);

        assertEq(
            Merkle.Hash.unwrap(outputsDisagree_.preAdvanceOutputs),
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceOutputs)
        );
        assertEq(
            Merkle.Hash.unwrap(outputsDisagree_.postAdvanceMachine),
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceMachineClaim)
        );
        assertEq(
            Merkle.Hash.unwrap(outputsDisagree_.postAdvanceOutputsClaim),
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceOutputsClaim)
        );
        assertEq(outputsDisagree_.inputIndex, waitingDivergence_.inputIndex);
    }

    /*
        Internal helper methods
    */

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    function createDivergence()
        internal
        pure
        returns (Partition.Divergence memory)
    {
        return
            Partition.Divergence(
                0,
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000001
            );
    }

    function createWaitingDivergence()
        internal
        pure
        returns (EpochHashSplit.WaitingDivergence memory)
    {
        return
            EpochHashSplit.WaitingDivergence(
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                0
            );
    }
}
