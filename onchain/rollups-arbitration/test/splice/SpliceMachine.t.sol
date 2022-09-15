// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/splice/SpliceMachine.sol";

contract TestSpliceMachine is Test {
    
    function setUp() public {
    }

    function test_createSplice() public {
        EpochHashSplit.MachineDisagree memory machineDisagree_ =
            EpochHashSplit.MachineDisagree(
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                0
            );
        
        SpliceMachine.WaitingSpliceClaim memory waitingSplice_ = 
            SpliceMachine.createSplice(machineDisagree_, 0);
        
        assertTrue(
            waitingSplice_.machineDisagree.inputIndex == machineDisagree_.inputIndex
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingSplice_.machineDisagree.preAdvanceMachine) == 
            Merkle.Hash.unwrap(machineDisagree_.preAdvanceMachine)
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingSplice_.machineDisagree.postAdvanceMachineClaim) == 
            Merkle.Hash.unwrap(machineDisagree_.postAdvanceMachineClaim)
        );
    }

    function test_spliceSupplyHash() public {
        SpliceMachine.WaitingAgreement memory waitingAgree_ =
            createWaitingAgreement();

        assertTrue(
            waitingAgree_.preSpliceData.machineDisagree.inputIndex == 
            0
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingAgree_.preSpliceData.machineDisagree.preAdvanceMachine) == 
            INITIAL_HASH
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingAgree_.preSpliceData.machineDisagree.postAdvanceMachineClaim) == 
            INITIAL_HASH
        );
    }

    function test_spliceAcceptClaim() public {
        SpliceMachine.WaitingAgreement memory waitingAgree_ =
            createWaitingAgreement();
        
        SpliceMachine.SpliceAgree memory spliceAgree_ =
            SpliceMachine.spliceAcceptClaim(waitingAgree_);
        
        assertTrue(
            Merkle.Hash.unwrap(spliceAgree_.postSpliceMachineHashClaim) == 
            Merkle.Hash.unwrap(waitingAgree_.postSpliceMachineHashClaim)
        );
        assertTrue(
            Merkle.Hash.unwrap(spliceAgree_.postAdvanceMachineClaim) == 
            Merkle.Hash.unwrap(waitingAgree_.preSpliceData.machineDisagree.postAdvanceMachineClaim)
        );
        
    }
    /*
        Internal helper methods
    */

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;
    
    function createWaitingAgreement() public pure returns (SpliceMachine.WaitingAgreement memory) {
        EpochHashSplit.MachineDisagree memory machineDisagree_ =
            EpochHashSplit.MachineDisagree(
                Merkle.Hash.wrap(INITIAL_HASH),
                Merkle.Hash.wrap(INITIAL_HASH),
                0
            );
        SpliceMachine.WaitingSpliceClaim memory waitingSplice_ = 
            SpliceMachine.createSplice(machineDisagree_, 0);
        Merkle.Hash postSpliceMachineHash_ = Merkle.Hash.wrap(INITIAL_HASH);
        SpliceMachine.WaitingAgreement memory waitingAgree_ =
            SpliceMachine.spliceSupplyHash(waitingSplice_, postSpliceMachineHash_);
        return waitingAgree_;
    }

}