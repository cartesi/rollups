// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../partition/Partition.sol";
import { Merkle } from "utils/Merkle.sol";

library EpochHashSplit {
    using Merkle for Merkle.Hash;

    struct WaitingSubhashes {
        Merkle.Hash preAdvanceMachine;
        Merkle.Hash preAdvanceOutputs;
        bytes32 postAdvanceEpochHashClaim;
        uint64 inputIndex;
    }

    struct WaitingDivergence {
        Merkle.Hash preAdvanceMachine;
        Merkle.Hash preAdvanceOutputs;
        Merkle.Hash postAdvanceMachineClaim;
        Merkle.Hash postAdvanceOutputsClaim;
        uint64 inputIndex;
    }

    /////// CHALLENGER DISAGREEMENT'S TYPES ///////

    struct MachineDisagree {
        Merkle.Hash preAdvanceMachine;
        Merkle.Hash postAdvanceMachineClaim;
        uint64 inputIndex;
    }

    struct OutputsDisagree {
        Merkle.Hash preAdvanceOutputs;
        Merkle.Hash postAdvanceMachine;
        Merkle.Hash postAdvanceOutputsClaim;
        uint64 inputIndex;
    }

    function createSplit(
        Partition.Divergence memory advanceStateDivergence,
        Merkle.Hash preAdvanceMachine,
        Merkle.Hash preAdvanceOutputs
    ) external pure returns(WaitingSubhashes memory) {
        require(
            preAdvanceOutputs.concatAndHash(preAdvanceMachine) ==
            advanceStateDivergence.beforeHash,
            "supplied hashes don't match epoch hash"
        );
        /*require(
            keccak256(abi.encode(Merkle.Hash.unwrap(preAdvanceOutputs), Merkle.Hash.unwrap(preAdvanceMachine))) ==
            advanceStateDivergence.beforeHash,
            "supplied hashes don't match epoch hash"
        );*/
        return WaitingSubhashes(
            preAdvanceMachine,
            preAdvanceOutputs,
            advanceStateDivergence.afterHash,
            advanceStateDivergence.divergencePoint
        );
    }

    function supplySubhashes(
        WaitingSubhashes memory waitingSubhashes,
        Merkle.Hash postAdvanceMachineClaim,
        Merkle.Hash postAdvanceOutputsClaim
    ) external pure returns(WaitingDivergence memory) {
        /*require(
            postAdvanceOutputsClaim.concatAndHash(postAdvanceMachineClaim) ==
            waitingSubhashes.postAdvanceEpochHashClaim,
            "supplied hashes don't match divergence epoch hash"
        );*/
        require(
            keccak256(abi.encode(Merkle.Hash.unwrap(postAdvanceOutputsClaim), Merkle.Hash.unwrap(postAdvanceMachineClaim))) ==
            waitingSubhashes.postAdvanceEpochHashClaim,
            "supplied hashes don't match divergence epoch hash"
        );

        return WaitingDivergence(
            waitingSubhashes.preAdvanceMachine,
            waitingSubhashes.preAdvanceOutputs,
            postAdvanceMachineClaim,
            postAdvanceOutputsClaim,
            waitingSubhashes.inputIndex
        );
    }

    function machineDisagree(
        WaitingDivergence memory waitingDivergence
    ) external pure returns(MachineDisagree memory) {
        return MachineDisagree(
            waitingDivergence.preAdvanceMachine,
            waitingDivergence.postAdvanceMachineClaim, //id: we dont necesarily have to put that is a claiming 
            waitingDivergence.inputIndex
        );
    }

    function outputsDisagree(
        WaitingDivergence memory waitingDivergence
    ) external pure returns(OutputsDisagree memory) {
        return OutputsDisagree(
            waitingDivergence.preAdvanceOutputs,
            waitingDivergence.postAdvanceMachineClaim,
            waitingDivergence.postAdvanceOutputsClaim,
            waitingDivergence.inputIndex
            //id: why do we need to pass the state of the machine also?
        );
    }
}
