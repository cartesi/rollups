// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Word } from "utils/Word.sol";

import "../partition/Partition.sol";
import "../epoch-hash-split/EpochHashSplit.sol";
import "./SpliceDataSource.sol";
import "./SpliceUtils.sol";

library SpliceMachine {
    struct WaitingSpliceClaim {
        EpochHashSplit.MachineDisagree machineDisagree;
        uint64 epochIndex; //we used to have a divergence, where a divegence point was stablished
    }

    struct WaitingAgreement {
        WaitingSpliceClaim preSpliceData;
        Merkle.Hash postSpliceMachineHashClaim;
    }

    struct SpliceAgree {
        Merkle.Hash postSpliceMachineHashClaim;
        Merkle.Hash postAdvanceMachineClaim;
    }

    function createSplice( //id: createMachineSplice
        EpochHashSplit.MachineDisagree memory machineDisagree,
        uint64 epochIndex
    ) external pure returns(WaitingSpliceClaim memory) {
        return WaitingSpliceClaim(machineDisagree, epochIndex);
    }

    function spliceSupplyHash(
        WaitingSpliceClaim memory waitingClaim,
        Merkle.Hash postSpliceMachineHash
    ) external pure returns(WaitingAgreement memory) {
        return WaitingAgreement(
            waitingClaim,
            postSpliceMachineHash
        );
    }

    function spliceAcceptClaim(
        WaitingAgreement memory claim
    ) external pure returns(SpliceAgree memory) {
        return SpliceAgree(
            claim.postSpliceMachineHashClaim,
            claim.preSpliceData.machineDisagree.postAdvanceMachineClaim
        );
    }


    /*function spliceRejectClaim(
        WaitingAgreement memory claim,
        SpliceDataSource dataSource,
        SpliceUtils.SpliceMachineProofs calldata proofs
    ) external pure {
        //require(check proofs);

        //validate input metdata and input hash matches the datasource.getinputHash
        
        // TODO
    }*/

    /*function spliceRejectClaim(
        WaitingAgreement memory claim
    ) external pure returns(SpliceDisagree memory) {
        return SpliceDisagree(
            claim.preSpliceData.machineDisagree.preAdvanceMachine,
            claim.postSpliceMachineHashClaim
        );
    }*/

    /*//Reject the claim, and return the pre-advance machine
    function spliceRejectClaim(
        WaitingAgreement memory claim
    ) external pure returns(Merkle.Hash memory) {
        return claim.preSpliceData.machineDisagree.preAdvanceMachine;
    }*/

    //Reject the claim, verify the proofs, and return the pre-advance machine

    //Reject the claim, verify the proofs, and return the pre-advance machine
    /*function spliceRejectClaim(
        WaitingAgreement memory claim,
        SpliceDataSource dataSource,
        SpliceUtils.SpliceMachineProofs calldata proofs
    ) external pure returns(Merkle.Hash memory) {
        return SpliceDisagree(
            claim.preSpliceData.machineDisagree.preAdvanceMachine,
            claim.postSpliceMachineHashClaim
        );
    }*/

}
