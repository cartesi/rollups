// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)
pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import "../../src/partition/PartitionEnum.sol";
import "../../src/splice/SpliceMachineEnum.sol";
import "../../src/two-party-arbitration/TwoPartyArbitration.sol";
import "../../src/utils/GameClockLib.sol";
import {Merkle} from "utils/Merkle.sol";

contract TestTwoPartyArbitration is Test {
    function setUp() public {}

    /*
        Two Party Arbitration Tests
    */
    function test_createArbitration() public {
        /* Init example variables */
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        /* Context creation */
        TwoPartyArbitration.Context memory context = createContext(args);

        GameClockLib.Timer memory manual_timer = GameClockLib
            .newTimerClaimerTurn(block.timestamp, args.timeAllowance);

        Partition.WaitingHash memory waitingHash = Partition.createPartition(
            0,
            args.numInputs,
            args.initialHash,
            args.claimedHash
        );

        /* Tagging data correctly */
        PartitionEnum.T memory waitingHashEnum = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        /* After untagging input partition, the tag is still waiting hash */
        PartitionEnum.T memory unboxed_state = TwoPartyArbitrationEnum
            .getInputPartitionVariant(context.state);

        compareWaitingHash(waitingHashEnum, unboxed_state);
        assertEq(context.arguments.challenger, PLAYER1_ADDRESS);
        assertEq(context.arguments.claimer, PLAYER2_ADDRESS);
        assertEq(context.arguments.maxCycle, 1);
        assertEq(context.timer.lastResume, manual_timer.lastResume);
        assertEq(
            context.timer.challengerAllowance,
            manual_timer.challengerAllowance
        );
        assertEq(context.timer.claimerAllowance, manual_timer.claimerAllowance);
        assertTrue(context.timer.turn == GameClockLib.Turn.Claimer);
    }

    function test_createArbitrationFuzzy(
        address challenger_,
        address claimer_,
        bytes32 initialHash_,
        bytes32 claimedHash_,
        uint256 timeAllowance_,
        uint64 maxCycle_,
        uint64 numInputs_,
        uint64 epochIndex_
    ) public {
        vm.assume(numInputs_ > 2);
        SpliceDataSource dataSource_;

        /* Context creation */
        TwoPartyArbitration.Context memory context = createContext(
            challenger_,
            claimer_,
            dataSource_,
            initialHash_,
            claimedHash_,
            epochIndex_,
            numInputs_,
            timeAllowance_,
            maxCycle_
        );

        GameClockLib.Timer memory manual_timer = GameClockLib
            .newTimerClaimerTurn(block.timestamp, timeAllowance_);

        Partition.WaitingHash memory waitingHash = Partition.createPartition(
            0,
            numInputs_,
            initialHash_,
            claimedHash_
        );

        /* Tagging data correctly */
        PartitionEnum.T memory waitingHashEnum = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        //After untagging input partition, the tag is still waiting hash
        PartitionEnum.T memory unboxed_state = TwoPartyArbitrationEnum
            .getInputPartitionVariant(context.state);

        compareWaitingHash(waitingHashEnum, unboxed_state);
        assertEq(context.timer.lastResume, manual_timer.lastResume);
        assertEq(
            context.timer.challengerAllowance,
            manual_timer.challengerAllowance
        );
        assertEq(context.timer.claimerAllowance, manual_timer.claimerAllowance);
        assertTrue(context.timer.turn == GameClockLib.Turn.Claimer);
    }

    /* Failing Timeout methods */
    /*function testFail_challengerWinByTimeout() public view {

        /* Init example variables 
        address challenger_ = PLAYER1_ADDRESS;
        address claimer_ = PLAYER2_ADDRESS;
        SpliceDataSource dataSource_;
        bytes32 initialHash_ = INITIAL_HASH;
        bytes32 claimedHash_ = CLAIMER_FINAL_HASH;
        uint256 timeAllowance_ = 0; //No time left for any turn
        uint64 maxCycle_ = 1;
        uint64 numInputs_ = 2;
        uint64 epochIndex_ = 0;

        /* Context creation 
        TwoPartyArbitration.Context memory context =
            createContext(
                challenger_,
                claimer_,
                dataSource_,
                initialHash_,
                claimedHash_,
                epochIndex_,
                numInputs_,
                timeAllowance_,
                maxCycle_
            );
        
        TwoPartyArbitration.challengerWinByTimeout(context);
    }*/

    function test_stateAdvanceSupplyIntermediateHash() public {
        /* Init example variables */
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();
        args.claimer = msg.sender;
        bytes32 replyHash_ = INTERMEDIATE_HASH;

        /* Context creation */
        TwoPartyArbitration.Context memory context = createContext(args);

        TwoPartyArbitration.Context
            memory intermediateContext = TwoPartyArbitration
                .stateAdvanceSupplyIntermediateHash(context, replyHash_);

        /* Tagging data correctly */
        PartitionEnum.T memory unboxed_state = TwoPartyArbitrationEnum
            .getInputPartitionVariant(intermediateContext.state);

        Partition.WaitingInterval memory unboxed_partition = PartitionEnum
            .getWaitingIntervalVariant(unboxed_state);

        compareContext(intermediateContext, context);
        assertTrue(
            intermediateContext.timer.turn == GameClockLib.Turn.Challenger
        );
        assertEq(unboxed_partition.intermediateHash, replyHash_);
    }

    function test_stateAdvanceSupplyIntermediateHashFuzzy(
        address challenger_,
        bytes32 initialHash_,
        bytes32 claimedHash_,
        bytes32 replyHash_,
        uint256 timeAllowance_,
        uint64 maxCycle_,
        uint64 numInputs_,
        uint64 epochIndex_
    ) public {
        vm.assume(numInputs_ > 2);
        vm.assume(timeAllowance_ > 1);
        address claimer_ = msg.sender;
        SpliceDataSource dataSource_;

        TwoPartyArbitration.Context memory context = createContext(
            challenger_,
            claimer_,
            dataSource_,
            initialHash_,
            claimedHash_,
            epochIndex_,
            numInputs_,
            timeAllowance_,
            maxCycle_
        );

        TwoPartyArbitration.Context
            memory intermediateContext = TwoPartyArbitration
                .stateAdvanceSupplyIntermediateHash(context, replyHash_);

        PartitionEnum.T memory unboxed_state = TwoPartyArbitrationEnum
            .getInputPartitionVariant(intermediateContext.state);

        Partition.WaitingInterval memory unboxed_partition = PartitionEnum
            .getWaitingIntervalVariant(unboxed_state);

        compareContext(intermediateContext, context);
        assertTrue(
            intermediateContext.timer.turn == GameClockLib.Turn.Challenger
        );
        assertEq(unboxed_partition.intermediateHash, replyHash_);
    }

    function test_stateAdvanceSupplyDivergenceInterval() public {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();
        bool agree_ = false;

        TwoPartyArbitration.Context
            memory context = createContextofWaitingInterval(
                args,
                INITIAL_POINT,
                FINAL_POINT
            );

        TwoPartyArbitration.Context memory advancedContext = TwoPartyArbitration
            .stateAdvanceSupplyDivergenceInterval(context, agree_);

        PartitionEnum.T memory advancedInputPartition = TwoPartyArbitrationEnum
            .getInputPartitionVariant(advancedContext.state);

        Partition.WaitingHash memory advancedWaitingHash = PartitionEnum
            .getWaitingHashVariant(advancedInputPartition);

        assertEq(advancedWaitingHash.agreePoint, INITIAL_POINT);
        assertTrue(advancedWaitingHash.disagreePoint != FINAL_POINT);
        assertEq(advancedWaitingHash.agreeHash, INITIAL_HASH);
        assertEq(advancedWaitingHash.disagreeHash, INTERMEDIATE_HASH);
    }

    /*function test_stateAdvanceSupplyDivergenceIntervalFuzzy(
        bytes32 initialHash_,
        bytes32 claimedHash_,
        address claimer_,
        bytes32 intermediateHash_,
        uint256 timeAllowance_,
        uint64 maxCycle_,
        uint64 numInputs_,
        uint64 epochIndex_
    ) public {
        address challenger_ = msg.sender;
        SpliceDataSource dataSource_;
        bool agree_ = false;
        vm.assume(numInputs_ > 2);
        vm.assume(timeAllowance_ > 1);

        TwoPartyArbitration.Context memory context = 
            createContextofWaitingInterval(
                claimer_,
                dataSource_,
                initialHash_,
                claimedHash_,
                intermediateHash_,
                epochIndex_,
                numInputs_,
                timeAllowance_,
                maxCycle_
            );

        
        TwoPartyArbitration.Context memory advancedContext =
            TwoPartyArbitration.stateAdvanceSupplyDivergenceInterval(
                context, agree_
            );

        PartitionEnum.T memory advancedInputPartition =
            TwoPartyArbitrationEnum.getInputPartitionVariant(
                advancedContext.state
            );

        Partition.WaitingHash memory advancedWaitingHash =
            PartitionEnum.getWaitingHashVariant(advancedInputPartition);

        assertEq(advancedWaitingHash.agreePoint, INITIAL_POINT);
        assertTrue(advancedWaitingHash.disagreePoint != FINAL_POINT);
        assertEq(advancedWaitingHash.agreeHash, initialHash_);
        assertEq(advancedWaitingHash.disagreeHash, intermediateHash_);
    }*/

    function testFail_stateAdvanceSupplyDivergenceInterval() public view {
        /*  stateAdvanceSupplyDivergenceInterval will fail if the context is waitingHash type */
        bool agree_ = false;
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .createArbitration(args);

        TwoPartyArbitration.stateAdvanceSupplyDivergenceInterval(
            context,
            agree_
        );
    }

    function test_stateAdvanceEndPartition() public {
        /* 
            Is important that the divergence point for the arguments match the epoch hash,
            in this case the divergence is the inital hash 
            Also, for Semisum result we need to have the new final point to be 2,\
            so we can end the partition ( 1 + (4-1)) /2 
        */
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();
        args.initialHash = bytes32(
            0x04cde762ef08b6b6c5ded8e8c4c0b3f4e5c9ad7342c88fcc93681b4588b73f05
        );
        uint64 finalPoint_ = 4;
        bool agree_ = false;
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000004
        );
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000005
        );

        TwoPartyArbitration.Context
            memory context = createContextofWaitingInterval(
                args,
                INITIAL_POINT,
                finalPoint_
            );

        TwoPartyArbitration.Context memory finalContext = TwoPartyArbitration
            .stateAdvanceEndPartition(
                context,
                agree_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );

        EpochHashSplitEnum.T memory epochHashSplit = TwoPartyArbitrationEnum
            .getEpochHashSplitVariant(finalContext.state);

        EpochHashSplit.WaitingSubhashes
            memory epochHashSplit_subhashes = EpochHashSplitEnum
                .getWaitingSubhashesVariant(epochHashSplit);

        assertEq(
            epochHashSplit_subhashes.postAdvanceEpochHashClaim,
            INTERMEDIATE_HASH
        );
        assertEq(
            Merkle.unwrap(epochHashSplit_subhashes.preAdvanceMachine),
            Merkle.unwrap(preAdvanceMachine_)
        );
        assertEq(
            Merkle.unwrap(epochHashSplit_subhashes.preAdvanceOutputs),
            Merkle.unwrap(preAdvanceOutputs_)
        );
    }

    function testFail_stateAdvanceEndPartition() public view {
        /* we wont have a split on epoch hash as divergence point doesnt match epoch hash root*/
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();
        uint64 finalPoint_ = 4;
        bool agree_ = false;
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000004
        );
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000005
        );

        TwoPartyArbitration.Context
            memory context = createContextofWaitingInterval(
                args,
                INITIAL_POINT,
                finalPoint_
            );

        TwoPartyArbitration.Context memory finalContext = TwoPartyArbitration
            .stateAdvanceEndPartition(
                context,
                agree_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );

        EpochHashSplitEnum.T memory epochHashSplit = TwoPartyArbitrationEnum
            .getEpochHashSplitVariant(finalContext.state);

        /*EpochHashSplit.WaitingSubhashes memory epochHashSplit_subhashes =
            EpochHashSplitEnum.getWaitingSubhashesVariant(
                epochHashSplit
            );*/
        EpochHashSplitEnum.getWaitingSubhashesVariant(epochHashSplit);
    }

    //
    // Epoch hash split
    //

    function test_splitSupplySubhashes() public {
        Merkle.Hash postAdvanceMachine_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000004
        );
        Merkle.Hash postAdvanceOutputs_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000005
        );

        /* claimer will submit postAdvanceMachine_ and postAdvanceOutputs_ 
        and should match divergence epoch hash, in this case we modify the divergence epoch hash
        to match the current variables */

        TwoPartyArbitration.Context
            memory epochHash = createCustomEpochHashSplit();

        TwoPartyArbitration.Context
            memory newEpochHashSplit = TwoPartyArbitration.splitSupplySubhashes(
                epochHash,
                postAdvanceMachine_,
                postAdvanceOutputs_
            );

        /* Untagging data */
        EpochHashSplitEnum.T memory epochHashSplit = TwoPartyArbitrationEnum
            .getEpochHashSplitVariant(newEpochHashSplit.state);

        EpochHashSplit.WaitingDivergence
            memory epochHashSplit_divergence = EpochHashSplitEnum
                .getWaitingDivergenceVariant(epochHashSplit);

        assertTrue(
            Merkle.Hash.unwrap(
                epochHashSplit_divergence.postAdvanceMachineClaim
            ) == Merkle.Hash.unwrap(postAdvanceMachine_)
        );
        assertTrue(
            Merkle.Hash.unwrap(
                epochHashSplit_divergence.postAdvanceOutputsClaim
            ) == Merkle.Hash.unwrap(postAdvanceOutputs_)
        );
    }

    /*function test_splitMachineDisagree() public {

    }*/
    /*
        Internal helper methods
    */

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    bytes32 constant CLAIMER_FINAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000001;

    bytes32 constant INTERMEDIATE_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000002;

    address constant PLAYER1_ADDRESS =
        0x0000000000000000000000000000000000000000;

    address constant PLAYER2_ADDRESS =
        0x0000000000000000000000000000000000000001;

    SpliceDataSource DATA_SOURCE;

    uint64 constant INITIAL_POINT = 1;
    uint64 constant FINAL_POINT = 5;

    function createWaitingHash(
        bytes32 initialHash,
        bytes32 claimerFinalHash,
        uint64 initialPoint,
        uint64 finalPoint
    ) internal pure returns (Partition.WaitingHash memory) {
        Partition.WaitingHash memory waitingHash = Partition.createPartition(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );

        return waitingHash;
    }

    function createWaitingInterval(
        bytes32 initialHash,
        bytes32 claimerFinalHash,
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 intermediateHash
    ) public pure returns (Partition.WaitingInterval memory) {
        Partition.WaitingHash memory waitingHash = createWaitingHash(
            initialHash,
            claimerFinalHash,
            initialPoint,
            finalPoint
        );

        return Partition.WaitingInterval(waitingHash, intermediateHash);
    }

    function createContextofWaitingInterval(
        TwoPartyArbitration.ArbitrationArguments memory args,
        uint64 initialPoint_,
        uint64 finalPoint_
    ) internal view returns (TwoPartyArbitration.Context memory) {
        //In this dispute, te sender must be the challenger.
        //So, we set the challenger to be the msg.sender.
        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval(
                args.initialHash,
                args.claimedHash,
                initialPoint_,
                finalPoint_,
                INTERMEDIATE_HASH
            );

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        return
            TwoPartyArbitration.Context(
                TwoPartyArbitration.ArbitrationArguments(
                    msg.sender,
                    args.claimer,
                    args.dataSource,
                    args.initialHash,
                    args.claimedHash,
                    args.epochIndex,
                    args.numInputs,
                    args.timeAllowance,
                    args.maxCycle
                ),
                GameClockLib.newTimerChallengerTurn(
                    block.timestamp,
                    args.timeAllowance
                ),
                TwoPartyArbitrationEnum.enumOfInputPartition(
                    enumWaitingInterval
                )
            );
    }

    function compareContext(
        TwoPartyArbitration.Context memory c1,
        TwoPartyArbitration.Context memory c2
    ) public {
        assertEq(c1.arguments.challenger, c2.arguments.challenger);
        assertEq(c1.arguments.claimer, c2.arguments.claimer);
        assertEq(c1.arguments.maxCycle, c2.arguments.maxCycle);
        assertEq(c1.timer.lastResume, c2.timer.lastResume);
        assertEq(c1.timer.challengerAllowance, c2.timer.challengerAllowance);
        assertEq(c1.timer.claimerAllowance, c2.timer.claimerAllowance);
    }

    function compareWaitingHash(
        PartitionEnum.T memory tpae1,
        PartitionEnum.T memory tpae2
    ) public {
        Partition.WaitingHash memory waitingHash1 = PartitionEnum
            .getWaitingHashVariant(tpae1);

        Partition.WaitingHash memory waitingHash2 = PartitionEnum
            .getWaitingHashVariant(tpae2);

        assertTrue(tpae1._tag == tpae2._tag);
        assertEq(waitingHash1.agreePoint, waitingHash2.agreePoint);
        assertEq(waitingHash1.disagreePoint, waitingHash2.disagreePoint);
        assertEq(waitingHash1.agreeHash, waitingHash2.agreeHash);
        assertEq(waitingHash1.disagreeHash, waitingHash2.disagreeHash);
    }

    function createContext(
        address challenger_,
        address claimer_,
        SpliceDataSource dataSource_,
        bytes32 initialHash_,
        bytes32 claimedHash_,
        uint64 epochIndex_,
        uint64 numInputs_,
        uint256 timeAllowance_,
        uint64 maxCycle_
    ) internal view returns (TwoPartyArbitration.Context memory) {
        TwoPartyArbitration.ArbitrationArguments
            memory arguments = TwoPartyArbitration.ArbitrationArguments(
                challenger_,
                claimer_,
                dataSource_,
                initialHash_,
                claimedHash_,
                epochIndex_,
                numInputs_,
                timeAllowance_,
                maxCycle_
            );

        return TwoPartyArbitration.createArbitration(arguments);
    }

    function createContext(
        TwoPartyArbitration.ArbitrationArguments memory args
    ) internal view returns (TwoPartyArbitration.Context memory) {
        return TwoPartyArbitration.createArbitration(args);
    }

    function createSampleArbitrationArguments()
        internal
        view
        returns (TwoPartyArbitration.ArbitrationArguments memory)
    {
        return
            TwoPartyArbitration.ArbitrationArguments(
                PLAYER1_ADDRESS,
                PLAYER2_ADDRESS,
                DATA_SOURCE,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH,
                0,
                3,
                2,
                1
            );
    }

    function createRandomSampleArbitrationArguments(
        address challenger_,
        address claimer_,
        bytes32 initialHash_,
        bytes32 claimedHash_,
        uint64 epochIndex_,
        uint64 numInputs_,
        uint256 timeAllowance_,
        uint64 maxCycle_
    ) internal returns (TwoPartyArbitration.ArbitrationArguments memory) {
        vm.assume(numInputs_ > 2);
        vm.assume(timeAllowance_ > 1);
        return
            TwoPartyArbitration.ArbitrationArguments(
                challenger_,
                claimer_,
                DATA_SOURCE,
                initialHash_,
                claimedHash_,
                epochIndex_,
                numInputs_,
                timeAllowance_,
                maxCycle_
            );
    }

    function createCustomEpochHashSplit()
        internal
        view
        returns (TwoPartyArbitration.Context memory)
    {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();
        args.initialHash = bytes32(
            0x04cde762ef08b6b6c5ded8e8c4c0b3f4e5c9ad7342c88fcc93681b4588b73f05
        );
        uint64 finalPoint_ = 4;
        bool agree_ = false;
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000004
        );
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(
            0x0000000000000000000000000000000000000000000000000000000000000005
        );
        bytes32 customEpochHashDivergence = bytes32(
            0x04cde762ef08b6b6c5ded8e8c4c0b3f4e5c9ad7342c88fcc93681b4588b73f05
        );

        /* claimer will submit postAdvanceMachine_ and postAdvanceOutputs_ 
        and should match divergence epoch hash, in this case we modify the divergence epoch hash
        to match the current variables */

        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval(
                args.initialHash,
                args.claimedHash,
                INITIAL_POINT,
                finalPoint_,
                customEpochHashDivergence
            );

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .Context(
                TwoPartyArbitration.ArbitrationArguments(
                    msg.sender,
                    msg.sender,
                    args.dataSource,
                    args.initialHash,
                    args.claimedHash,
                    args.epochIndex,
                    args.numInputs,
                    args.timeAllowance,
                    args.maxCycle
                ),
                GameClockLib.newTimerChallengerTurn(
                    block.timestamp,
                    args.timeAllowance
                ),
                TwoPartyArbitrationEnum.enumOfInputPartition(
                    enumWaitingInterval
                )
            );

        return (
            TwoPartyArbitration.stateAdvanceEndPartition(
                context,
                agree_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            )
        );
    }
}
