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
import "../../src/epoch-hash-split/EpochHashSplit.sol";
import "../../src/epoch-hash-split/EpochHashSplitEnum.sol";
import "../../src/partition/Partition.sol";
import { Merkle } from "utils/Merkle.sol";


contract TestEpochHashSplitEnum is Test {
    function setUp() public {
    }

    function test_enumOfWaitingSubhashes() public {
        Partition.Divergence memory divergence_ = createDivergence();
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        divergence_.beforeHash = bytes32(0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

        EpochHashSplit.WaitingSubhashes memory waitingSubhashes_ = 
            EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );
        EpochHashSplitEnum.T memory enumWaitingSubhashes_ =
            EpochHashSplitEnum.enumOfWaitingSubhashes(waitingSubhashes_);

        assertTrue(
            EpochHashSplitEnum.isWaitingSubhashesVariant(enumWaitingSubhashes_)
        );
    }

    function test_enumOfWaitingDivergence() public {
        EpochHashSplit.WaitingDivergence memory waitingDivergence_ = 
            createWaitingDivergence();

        EpochHashSplitEnum.T memory enumWaitingDivergence_ =
            EpochHashSplitEnum.enumOfWaitingDivergence(waitingDivergence_);

        assertTrue(
            EpochHashSplitEnum.isWaitingDivergenceVariant(enumWaitingDivergence_)
        );    
    }

    function testFail_isWaitingSubhashesVariant() public {
        EpochHashSplit.WaitingDivergence memory waitingDivergence_ = 
            createWaitingDivergence();

        EpochHashSplitEnum.T memory enumWaitingDivergence_ =
            EpochHashSplitEnum.enumOfWaitingDivergence(waitingDivergence_);

        assertTrue(
            EpochHashSplitEnum.isWaitingSubhashesVariant(enumWaitingDivergence_)
        );
    }

    function testFail_isWaitingDivergenceVariant() public {
        Partition.Divergence memory divergence_ = createDivergence();
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        divergence_.beforeHash = bytes32(0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

        EpochHashSplit.WaitingSubhashes memory waitingSubhashes_ = 
            EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );
        EpochHashSplitEnum.T memory enumWaitingSubhashes_ =
            EpochHashSplitEnum.enumOfWaitingSubhashes(waitingSubhashes_);
        
        assertTrue(
            EpochHashSplitEnum.isWaitingDivergenceVariant(enumWaitingSubhashes_)
        );    
    }

    function test_getWaitingSubhashesVariant() public {
        Partition.Divergence memory divergence_ = createDivergence();
        Merkle.Hash preAdvanceMachine_ = Merkle.Hash.wrap(INITIAL_HASH);
        Merkle.Hash preAdvanceOutputs_ = Merkle.Hash.wrap(INITIAL_HASH);
        divergence_.beforeHash = bytes32(0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5);

        EpochHashSplit.WaitingSubhashes memory waitingSubhashes_ = 
            EpochHashSplit.createSplit(
                divergence_,
                preAdvanceMachine_,
                preAdvanceOutputs_
            );
        EpochHashSplitEnum.T memory enumWaitingSubhashes_ =
            EpochHashSplitEnum.enumOfWaitingSubhashes(waitingSubhashes_);
        EpochHashSplit.WaitingSubhashes memory recoveredWaitingSubhashes_ =
            EpochHashSplitEnum.getWaitingSubhashesVariant(enumWaitingSubhashes_);

        assertTrue(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceMachine) == 
            Merkle.Hash.unwrap(recoveredWaitingSubhashes_.preAdvanceMachine)
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingSubhashes_.preAdvanceOutputs) == 
            Merkle.Hash.unwrap(recoveredWaitingSubhashes_.preAdvanceOutputs)
        );
        assertTrue(
            waitingSubhashes_.postAdvanceEpochHashClaim == 
            recoveredWaitingSubhashes_.postAdvanceEpochHashClaim
        );
        assertTrue(
            waitingSubhashes_.inputIndex == recoveredWaitingSubhashes_.inputIndex
        );
    }

    function test_getWaitingDivergenceVariant() public {
        EpochHashSplit.WaitingDivergence memory waitingDivergence_ = 
            createWaitingDivergence();

        EpochHashSplitEnum.T memory enumWaitingDivergence_ =
            EpochHashSplitEnum.enumOfWaitingDivergence(waitingDivergence_);
        EpochHashSplit.WaitingDivergence memory recoveredWaitingDivergence_ =
            EpochHashSplitEnum.getWaitingDivergenceVariant(enumWaitingDivergence_);
        
        assertTrue(
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceMachine) == 
            Merkle.Hash.unwrap(recoveredWaitingDivergence_.preAdvanceMachine)
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingDivergence_.preAdvanceOutputs) == 
            Merkle.Hash.unwrap(recoveredWaitingDivergence_.preAdvanceOutputs)
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceMachineClaim) == 
            Merkle.Hash.unwrap(recoveredWaitingDivergence_.postAdvanceMachineClaim)
        );
        assertTrue(
            Merkle.Hash.unwrap(waitingDivergence_.postAdvanceOutputsClaim) == 
            Merkle.Hash.unwrap(recoveredWaitingDivergence_.postAdvanceOutputsClaim)
        );
        assertTrue(waitingDivergence_.inputIndex == recoveredWaitingDivergence_.inputIndex);
    }


    /*
        Internal helper methods
    */

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    function createDivergence() internal pure returns (Partition.Divergence memory){
        return Partition.Divergence(
            0,
            0x0000000000000000000000000000000000000000000000000000000000000000,
            0x0000000000000000000000000000000000000000000000000000000000000001
        );
    }

    function createWaitingDivergence() internal pure returns(EpochHashSplit.WaitingDivergence memory){
        return EpochHashSplit.WaitingDivergence(
            Merkle.Hash.wrap(INITIAL_HASH),
            Merkle.Hash.wrap(INITIAL_HASH),
            Merkle.Hash.wrap(INITIAL_HASH),
            Merkle.Hash.wrap(INITIAL_HASH),
            0
        );
    }

}