// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../partition/Partition.sol";
import "../partition/PartitionEnum.sol";

import "../splice/SpliceDataSource.sol";
import "../splice/SpliceOutputs.sol";
import "../splice/SpliceMachine.sol";
import "../splice/SpliceMachineEnum.sol";

import "utils/GameClockLib.sol";

import "./TwoPartyArbitrationEnum.sol";


library TwoPartyArbitration {
    using TwoPartyArbitrationEnum for TwoPartyArbitrationEnum.T;

    using TwoPartyArbitrationEnum for PartitionEnum.T;
    using PartitionEnum for PartitionEnum.T;

    using PartitionEnum for Partition.WaitingHash;
    using Partition for Partition.WaitingHash;

    using PartitionEnum for Partition.WaitingInterval;
    using Partition for Partition.WaitingInterval;

    using TwoPartyArbitrationEnum for EpochHashSplitEnum.T;
    using EpochHashSplitEnum for EpochHashSplitEnum.T;

    using EpochHashSplitEnum for EpochHashSplit.WaitingSubhashes;
    using EpochHashSplit for EpochHashSplit.WaitingSubhashes;

    using EpochHashSplitEnum for EpochHashSplit.WaitingDivergence;
    using EpochHashSplit for EpochHashSplit.WaitingDivergence;

    using TwoPartyArbitrationEnum for SpliceMachineEnum.T;
    using SpliceMachineEnum for SpliceMachineEnum.T;

    using SpliceMachineEnum for SpliceMachine.WaitingSpliceClaim;
    using SpliceMachine for SpliceMachine.WaitingSpliceClaim;

    using GameClockLib for GameClockLib.Timer;

    enum Status {ChallengerWon, ClaimerWon}

    struct ArbitrationArguments {
        address challenger;
        address claimer;
        SpliceDataSource dataSource;
        bytes32 initialHash;
        bytes32 claimedHash;
        uint64 epochIndex;
        uint64 numInputs;
        uint256 timeAllowance;
        uint64 maxCycle;
    }

    struct Context {
        ArbitrationArguments arguments;
        GameClockLib.Timer timer;
        TwoPartyArbitrationEnum.T state;
    }

    modifier onlyChallenger(Context memory context) {
        require(
            msg.sender == context.arguments.challenger,
            "msg.sender is not challenger"
        );
        _;
    }

    modifier onlyClaimer(Context memory context) {
        require(
            msg.sender == context.arguments.claimer,
            "msg.sender is not claimer"
        );
        _;
    }

   function createArbitration(
       ArbitrationArguments memory arguments
    )
        external
        view
        returns(Context memory)
    {
        return Context(
            arguments,

            GameClockLib.newTimerClaimerTurn(
                block.timestamp,
                arguments.timeAllowance
            ), //TIME ALLOWANCE MUST BE BIGGER THAN 0 ON A NEW TURN AND MAYBE SET A DECENT TIME , ON TIMER LIB

            Partition.createPartition(
                0,
                arguments.numInputs,
                arguments.initialHash,
                arguments.claimedHash
            )
            .enumOfWaitingHash()
            .enumOfInputPartition()
        );
    }

    //
    // Timeout methods
    //

    function challengerWinByTimeout(
        Context memory context
    )
        external
        view
    {
        require(
            !context.timer.challengerHasTimeLeft(block.timestamp),
            "claimer has time left"
        );
    }

    function claimerWinByTimeout(
        Context memory context
    )
        external
        view
    {
        require(
            !context.timer.challengerHasTimeLeft(block.timestamp),
            "challenger has time left"
        );
    }


    //
    // Advance State Partition
    //

    function stateAdvanceSupplyIntermediateHash(
        Context memory context,
        bytes32 replyHash
    )
        external
        view
        onlyClaimer(context)
        returns(Context memory)
    {
        return Context(
            context.arguments,

            context.timer.claimerPassTurn(block.timestamp),

            context
                .state
                .getInputPartitionVariant()
                .getWaitingHashVariant()
                .supplyIntermediateHash(replyHash)
                .enumOfWaitingInterval()
                .enumOfInputPartition()
        );
    }

    function stateAdvanceSupplyDivergenceInterval(
        Context memory context,
        bool agree
    )
        external
        view
        onlyChallenger(context)
        returns(Context memory)
    {
        return Context(
            context.arguments,

            context.timer.challengerPassTurn(block.timestamp),

            context
                .state
                .getInputPartitionVariant()
                .getWaitingIntervalVariant()
                .supplyDivergenceInterval(agree)
                .enumOfWaitingHash()
                .enumOfInputPartition()
        );
    }


    function stateAdvanceEndPartition(
        Context memory context,
        bool agree,
        Merkle.Hash preAdvanceMachine,
        Merkle.Hash preAdvanceOutputs
    )
        external
        view
        onlyChallenger(context)
        returns(Context memory)
    {
        Partition.Divergence memory divergence = context
            .state
            .getInputPartitionVariant()
            .getWaitingIntervalVariant()
            .endPartition(agree);

        return Context(
            context.arguments,

            context.timer.challengerPassTurn(block.timestamp),

            EpochHashSplit
                .createSplit(divergence, preAdvanceMachine, preAdvanceOutputs)
                .enumOfWaitingSubhashes()
                .enumOfEpochHashSplit()
        );
    }


    //
    // Epoch hash split
    //

    function splitSupplySubhashes(
        Context memory context,
        Merkle.Hash postAdvanceMachineClaim,
        Merkle.Hash postAdvanceOutputsClaim
    )
        external
        view
        onlyClaimer(context)
        returns(Context memory)
    {
        return Context(
            context.arguments,

            context.timer.claimerPassTurn(block.timestamp),

            context
                .state
                .getEpochHashSplitVariant()
                .getWaitingSubhashesVariant()
                .supplySubhashes(postAdvanceMachineClaim, postAdvanceOutputsClaim)
                .enumOfWaitingDivergence()
                .enumOfEpochHashSplit()
        );
    }

    function splitMachineDisagree(
        Context memory context
    )
        external
        view
        onlyChallenger(context)
        returns(Context memory)
    {
        EpochHashSplit.MachineDisagree memory machineDisagree = context
            .state
            .getEpochHashSplitVariant()
            .getWaitingDivergenceVariant()
            .machineDisagree();

        return Context(
            context.arguments,

            context.timer.challengerPassTurn(block.timestamp),

            SpliceMachine
                .createSplice(machineDisagree, context.arguments.epochIndex)
                .enumOfWaitingSpliceClaim()
                .enumOfMachineSplice()
        );
    }

    function challengerWinsOutputsSplice(
        Context memory context,
        SpliceOutputs.SpliceOutputsProofs calldata proofs
    )
        external
        view
        onlyChallenger(context)
    {
        EpochHashSplit.OutputsDisagree memory outputsDisagree = context
            .state
            .getEpochHashSplitVariant()
            .getWaitingDivergenceVariant()
            .outputsDisagree();

        SpliceDataSource.AddressSpace memory addressSpace =
            context.arguments.dataSource.getAddressSpace();

        Merkle.Hash postAdvanceOutputsHash = SpliceOutputs.spliceOutputs(
            outputsDisagree.postAdvanceMachine,
            outputsDisagree.preAdvanceOutputs,
            proofs,
            addressSpace,
            outputsDisagree.inputIndex
        );

        // If spliced outputs don't match claim, challanger has won.
        require(
            !Merkle.eq(postAdvanceOutputsHash, outputsDisagree.preAdvanceOutputs),
            "Outputs splice matches claim"
        );
    }




}
