// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.13;

import "forge-std/Test.sol";

import "../../src/partition/PartitionEnum.sol";
import "../../src/splice/SpliceMachineEnum.sol";
import "../../src/two-party-arbitration/TwoPartyArbitrationEnum.sol";
import "../../src/two-party-arbitration/TwoPartyArbitration.sol";

import "./TwoPartyArbitration.t.sol";

import {Merkle} from "utils/Merkle.sol";

contract TestTwoPartyArbitrationEnum is Test {
    using Merkle for Merkle.Hash;

    function setUp() public {}

    /*
     * @dev Tests the `InputPartition` variant.
     */

    function test_enumOfInputPartition() public {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .createArbitration(args);

        assertTrue(
            TwoPartyArbitrationEnum.isInputPartitionVariant(context.state)
        );
    }

    function testFail_isInputPartitionVariant() public {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .createArbitration(args);

        assertFalse(
            TwoPartyArbitrationEnum.isInputPartitionVariant(context.state)
        );
    }

    function test_getInputPartitionVariant() public {
        TwoPartyArbitration.Context
            memory context = createInputPartitionVariant();

        PartitionEnum.T memory enumWaitingInterval = TwoPartyArbitrationEnum
            .getInputPartitionVariant(context.state);

        assertTrue(PartitionEnum.isWaitingIntervalVariant(enumWaitingInterval));
    }

    /*
     * @dev Tests the `EpochHashSplit` variant.
     */

    function test_enumOfEpochHashSplit() public {
        TwoPartyArbitration.Context
            memory finalContext = createEpochHashSplit();

        assertTrue(
            TwoPartyArbitrationEnum.isEpochHashSplitVariant(finalContext.state)
        );
    }

    function testFail_isEpochHashSplitVariant() public {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .createArbitration(args);

        assertTrue(
            TwoPartyArbitrationEnum.isEpochHashSplitVariant(context.state)
        );
    }

    function test_getEpochHashSplitVariant() public {
        TwoPartyArbitration.Context
            memory finalContext = createEpochHashSplit();

        EpochHashSplitEnum.T memory epochHashSplit = TwoPartyArbitrationEnum
            .getEpochHashSplitVariant(finalContext.state);

        EpochHashSplit.WaitingSubhashes
            memory epochHashSplit_subhashes = EpochHashSplitEnum
                .getWaitingSubhashesVariant(epochHashSplit);

        assertEq(
            epochHashSplit_subhashes.postAdvanceEpochHashClaim,
            INTERMEDIATE_HASH
        );
    }

    /*
     * @dev Tests the `MachineSplice` variant.
     */

    function test_enumOfMachineSplice() public {
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

        TwoPartyArbitration.Context memory machineSplice = TwoPartyArbitration
            .splitMachineDisagree(newEpochHashSplit);

        assertTrue(
            TwoPartyArbitrationEnum.isMachineSpliceVariant(machineSplice.state)
        );
    }

    function testFail_isMachineSpliceVariant() public {
        TwoPartyArbitration.ArbitrationArguments
            memory args = createSampleArbitrationArguments();

        TwoPartyArbitration.Context memory context = TwoPartyArbitration
            .createArbitration(args);

        assertTrue(
            TwoPartyArbitrationEnum.isMachineSpliceVariant(context.state)
        );
    }

    function test_getMachineSpliceVariant() public view {
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

        TwoPartyArbitration.Context
            memory contextMachineSplice = TwoPartyArbitration
                .splitMachineDisagree(newEpochHashSplit);

        TwoPartyArbitrationEnum.getMachineSpliceVariant(
            contextMachineSplice.state
        );

        /*SpliceMachineEnum.T memory machineSplice2 =
            TwoPartyArbitrationEnum.getMachineSpliceVariant(
                contextMachineSplice.state
            );*/
        // TODO: COMPARE THIS BUT WE HAVE TO UNBOX IT FIRST
        //assertTrue(contextMachineSplice.state == machineSplice2 )
        /*assertTrue(machineSplice2.preAdvanceMachine == postAdvanceMachine_);
        assertTrue(machineSplice2.postAdvanceMachineClaim == postAdvanceOutputs_);*/
    }

    /*
     * @dev Tests the `InstructionPartition` variant.
     */
    /*
    function test_enumOfInstructionPartition() public {

    }

    function testFail_isInstructionPartitionVariant() public {
        
    }
    
    function test_getInstructionPartitionVariant() public {
        
    }*/

    /*
     * @dev Tests the `ProveMemory` variant.
     */
    /*
    function test_enumOfProveMemory() public {

    }

    function testFail_isProveMemoryVariant() public {
        
    }
    
    function test_ggetProveMemoryVariant() public {
        
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

    function createInputPartitionVariant()
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

        TwoPartyArbitration.Context
            memory context = createContextofWaitingInterval(
                args,
                INITIAL_POINT,
                finalPoint_
            );

        return context;
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
                    args.challenger, //just changed from msg.sender to args.challenger
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

    function createEpochHashSplit()
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

        TwoPartyArbitration.Context
            memory context = createContextofWaitingInterval(
                args,
                INITIAL_POINT,
                finalPoint_
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
