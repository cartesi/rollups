// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/splice/SpliceMachine.sol";
import "../../src/splice/SpliceMachineEnum.sol";
import {Merkle} from "utils/Merkle.sol";

contract TestSpliceMachineEnum is Test {
    function setUp() public {}

    //
    // `WaitingSpliceClaim` methods
    //

    function test_enumOfWaitingSpliceClaim() public {
        SpliceMachine.WaitingSpliceClaim
            memory waitingSpliceClaim = createWaitingSplice();

        SpliceMachineEnum.T memory enumWaitingSpliceClaim = SpliceMachineEnum
            .enumOfWaitingSpliceClaim(waitingSpliceClaim);

        assertTrue(
            SpliceMachineEnum.isWaitingSpliceClaimVariant(
                enumWaitingSpliceClaim
            )
        );
    }

    function testFail_isWaitingSpliceClaimVariant() public {
        SpliceMachine.WaitingAgreement
            memory waitingAgreemnt = createWaitingAgreement();

        SpliceMachineEnum.T memory enumWaitingAgreement = SpliceMachineEnum
            .enumOfWaitingAgreement(waitingAgreemnt);

        assertTrue(
            SpliceMachineEnum.isWaitingSpliceClaimVariant(enumWaitingAgreement)
        );
    }

    function test_getWaitingSpliceClaimVariant() public {
        SpliceMachine.WaitingSpliceClaim
            memory waitingSpliceClaim = createWaitingSplice();

        SpliceMachineEnum.T memory enumWaitingSpliceClaim = SpliceMachineEnum
            .enumOfWaitingSpliceClaim(waitingSpliceClaim);

        SpliceMachine.WaitingSpliceClaim
            memory recoveredWaitingSpliceClaim = SpliceMachineEnum
                .getWaitingSpliceClaimVariant(enumWaitingSpliceClaim);

        assertTrue(
            recoveredWaitingSpliceClaim.machineDisagree.inputIndex ==
                waitingSpliceClaim.machineDisagree.inputIndex
        );
        assertTrue(
            Merkle.Hash.unwrap(
                recoveredWaitingSpliceClaim.machineDisagree.preAdvanceMachine
            ) ==
                Merkle.Hash.unwrap(
                    waitingSpliceClaim.machineDisagree.preAdvanceMachine
                )
        );
        assertTrue(
            Merkle.Hash.unwrap(
                recoveredWaitingSpliceClaim
                    .machineDisagree
                    .postAdvanceMachineClaim
            ) ==
                Merkle.Hash.unwrap(
                    waitingSpliceClaim.machineDisagree.postAdvanceMachineClaim
                )
        );
        assertTrue(
            recoveredWaitingSpliceClaim.epochIndex ==
                waitingSpliceClaim.epochIndex
        );
    }

    //
    // `WaitingAgreement` methods
    //

    function test_enumOfWaitingAgreement() public {
        SpliceMachine.WaitingAgreement
            memory waitingAgreement = createWaitingAgreement();

        SpliceMachineEnum.T memory enumWaitingAgreement = SpliceMachineEnum
            .enumOfWaitingAgreement(waitingAgreement);

        assertTrue(
            SpliceMachineEnum.isWaitingAgreementVariant(enumWaitingAgreement)
        );
    }

    function testFail_isWaitingAgreementVariant() public {
        SpliceMachine.WaitingSpliceClaim
            memory waitingSpliceClaim = createWaitingSplice();

        SpliceMachineEnum.T memory enumWaitingSpliceClaim = SpliceMachineEnum
            .enumOfWaitingSpliceClaim(waitingSpliceClaim);

        assertTrue(
            SpliceMachineEnum.isWaitingAgreementVariant(enumWaitingSpliceClaim)
        );
    }

    function test_getWaitingAgreementVariant() public {
        SpliceMachine.WaitingAgreement
            memory waitingAgreement = createWaitingAgreement();

        SpliceMachineEnum.T memory enumWaitingAgreement = SpliceMachineEnum
            .enumOfWaitingAgreement(waitingAgreement);

        SpliceMachine.WaitingAgreement
            memory recoveredWaitingAgreement = SpliceMachineEnum
                .getWaitingAgreementVariant(enumWaitingAgreement);

        assertTrue(
            Merkle.Hash.unwrap(
                recoveredWaitingAgreement.postSpliceMachineHashClaim
            ) == Merkle.Hash.unwrap(waitingAgreement.postSpliceMachineHashClaim)
        );
        assertTrue(
            recoveredWaitingAgreement
                .preSpliceData
                .machineDisagree
                .inputIndex ==
                waitingAgreement.preSpliceData.machineDisagree.inputIndex
        );
        assertTrue(
            Merkle.Hash.unwrap(
                recoveredWaitingAgreement
                    .preSpliceData
                    .machineDisagree
                    .preAdvanceMachine
            ) ==
                Merkle.Hash.unwrap(
                    waitingAgreement
                        .preSpliceData
                        .machineDisagree
                        .preAdvanceMachine
                )
        );
    }

    /*
        Internal helper methods
    */

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    function createWaitingSplice()
        internal
        pure
        returns (SpliceMachine.WaitingSpliceClaim memory)
    {
        EpochHashSplit.MachineDisagree memory machineDisagree_ = EpochHashSplit
            .MachineDisagree(
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                0
            );

        return (SpliceMachine.createSplice(machineDisagree_, 0));
    }

    function createWaitingAgreement()
        internal
        pure
        returns (SpliceMachine.WaitingAgreement memory)
    {
        EpochHashSplit.MachineDisagree memory machineDisagree_ = EpochHashSplit
            .MachineDisagree(
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                0
            );
        SpliceMachine.WaitingSpliceClaim memory waitingSplice_ = SpliceMachine
            .createSplice(machineDisagree_, 0);
        Merkle.Hash postSpliceMachineHash_ = Merkle.Hash.wrap(INITIAL_HASH);
        SpliceMachine.WaitingAgreement memory waitingAgree_ = SpliceMachine
            .spliceSupplyHash(waitingSplice_, postSpliceMachineHash_);
        return waitingAgree_;
    }
}
