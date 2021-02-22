// Copyright (C) 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: GPL-3.0-only
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.


/// @title Interface DescartesV2 contract
pragma solidity ^0.7.0;

contract DescartesV2 {


    // inputs received during InputAccumulation will be included in the
    // current epoch. Inputs received while WaitingClaims or ChallengesInProgress
    // are accumulated for N + 1
    enum Phase { InputAccumulation, AwaitingConsensus, AwaitingDisputes }
    /**
    * // TODO: this is quite ugly, once agreed upon we can beautify it
    *
    *                All claims agreed OR challenge period ended
    *                functions: claim() or finalizeEpoch()
    *          +--------------------------------------------------+
    *          |                                                  |
    * +--------v-----------+   new input after IPAD     +---------+----------+
    * |                    +--------------------------->+                    |
    * | Input Accumulation |   firt claim after IPAD    | Awaiting Consensus |
    * |                    +--------------------------->+                    |
    * +-+------------------+                            +-----------------+--+
    *   ^                                                                 ^  |
    *   |                                              dispute resolved   |  |
    *   |  dispute resolved                            before challenge   |  |
    *   |  after challenge     +--------------------+  period ended       |  |
    *   |  period ended        |                    +---------------------+  |
    *   +----------------------+  Awaiting Dispute  |                        |
    *                          |                    +<-----------------------+
    *                          +--------------------+    conflicting claim
    **/

    struct State {
        bytes32 epochHash; // epoch hash to being suggested for epochs merkle tree
        uint256 timestamp; // timestamp of claim submission

        bytes32 attestatorMask; // mask of all addresses that agreed with claim

        // address claimer and address challenger
        // will probably live in the dispute resolution contract
        // they only exist if there is a dispute

        Phase phase; // phase of current state
    }


    // DescartesV2 only keeps the summary of all epochs, using the Merkle Hash
    bytes32 epochsMRH; // Merkle root hash of epochs
    /*
    * Suggested epochsMRH structure
    *                                    +-----+
    *                                    | MRH |
    *                                    +--+--+
    *                                       |
    *                           +-------+   |  +-------+
    *                           |Epoch 1|---+--|Epoch 2|
    *                           +---+---+      +-------+
    *                               |
    *                   +-----+     |    +-----+
    *                   |I/O 1|-----+----|I/0 2|
    *                   +--+--+          +-----+
    *                      |
    *          +-------+   |   +---------+
    *          |Input 1|---+---|Output MR|
    *          +-------+       +---------+
    *                               |
    *                  +--------+   |     +--------+
    *                  |Output 0|---+-----|Output 1|
    *                  +--------+         +--------+
    *
    *
    * TODO: discuss what here has to be accessed frequently by the chain
    * to weight the tradeoffs between storage/processing
    */

    // max number of machine cycles accross every possible dapp computation
    uint64 maxCycle;

    // contract responsible for input/output management
    address IOManager;

    // contract that manages the validators
    address validatorManager;

    // contract that manages this dapps disputes
    address disputeManager;

    // this might move to IOManager as well
    // new inputs after inputWindow get accumulated for next epoch
    uint64 inputWindow;  // time in seconds each epoch waits for inputs
                         // if there is a challenge/invalid state, the inputs
                         // get accumulated to the next epoch

    uint64 challengePeriod; // after challengePeriod the current state is finalized

    State state; // current state

// THIS BELONGS TO IOManager
//    struct Output {
//        bytes32 outputHash; // hash of output - includes destination, payload, input hash and maybe highestGas
//
//        // THIS MOVES TO THE BRIDGE CONTRACT
//        bool executed; // true if executed without reverting
//        bytes32[] dependencies; // outputs that this output depends on
//                                // can only be executed if all of them have been
//                                // properly executed (executed == true)
//    }


    function claim(bytes32 _epochHash) public {
        // check if its the first claim
        // if not, if it agrees with previous claim update the attestorMask
        // if attestors mask == consensus mask (validatorManager.getConsensusMask?)
        //      state change from awaiting consensus -> accumulating input
        // if it doesnt agree instantiates a dispute
    }

    function finalizeEpoch() public {
        // anyone can call

        // require phase == WaitingClaims && claim deadline passed
        // ||
        // require phase == ChallengeInProgress &&
        //                  (challenge deadline passed || there is a winner)
    }

    // descartesv2 gets notified when there is a new input
    // only by input contract
    function notifyInput() public {

    }
    // THIS GOES TO IOContract
    // function executeOutput(
    //     Epoch _outputEpoch,
    //     bytes32 _outputHash,
    //     uint256 _outputIndex, // index of the output leaf
    //     bytes32[] _inputSiblings, // Siblings to prove inputs merkle root hash
    //     bytes32[] _outputSiblings, // Siblings to prove outputs merkle root hash
    //     bytes32 _payload, // should be contained in outputHash?
    //     address _destination // should be contained in outputHash?
    // ) returns (bool) {
    //     if (executed[_outputHash]) return true;
    // }
}
