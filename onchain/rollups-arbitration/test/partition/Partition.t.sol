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
import "../../src/partition/Partition.sol";

contract TestPartition is Test {

    function setUp() public {}

    //
    // Partition tests
    //

    function test_createPartition() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                2,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        assertEq(waitingHash.agreePoint, 0);
        assertEq(waitingHash.disagreePoint, 2);
        assertEq(waitingHash.agreeHash, INITIAL_HASH);
        assertEq(waitingHash.disagreeHash, CLAIMER_FINAL_HASH);
    }

    function testFail_createPartition1() public pure {
        Partition.createPartition(
            1,
            0,
            INITIAL_HASH,
            CLAIMER_FINAL_HASH
        );
    }

    function testFail_createPartition2() public pure {
        Partition.createPartition(
            1,
            1,
            INITIAL_HASH,
            CLAIMER_FINAL_HASH
        );
    }

    function testFail_createPartition3() public pure {
        Partition.createPartition(
            0,
            1,
            INITIAL_HASH,
            CLAIMER_FINAL_HASH
        );
    }

    function test_supplyIntermediateHash() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                4,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        Partition.WaitingInterval memory waitingInterval =
            Partition.WaitingInterval(waitingHash, INTERMEDIATE_HASH);

        assertEq(waitingInterval.waitingHash.agreePoint, 0);
        assertEq(waitingInterval.waitingHash.disagreePoint, 4);
        assertEq(waitingInterval.waitingHash.agreeHash, INITIAL_HASH);
        assertEq(waitingInterval.waitingHash.disagreeHash, CLAIMER_FINAL_HASH);
        assertEq(waitingInterval.intermediateHash, INTERMEDIATE_HASH);
    }

    function test_supplyDivergenceIntervalAgree() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                4,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        Partition.WaitingInterval memory waitingInterval =
            Partition.supplyIntermediateHash(waitingHash, INTERMEDIATE_HASH);

        Partition.WaitingHash memory nextWaitingHash =
            Partition.supplyDivergenceInterval(waitingInterval, true);

        assertEq(2, nextWaitingHash.agreePoint);
        assertEq(4, nextWaitingHash.disagreePoint);
        assertEq(INTERMEDIATE_HASH, nextWaitingHash.agreeHash);
        assertEq(CLAIMER_FINAL_HASH, nextWaitingHash.disagreeHash);
    }

    function test_supplyDivergenceIntervalDisagree() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                4,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        Partition.WaitingInterval memory waitingInterval =
            Partition.supplyIntermediateHash(waitingHash, INTERMEDIATE_HASH);

        Partition.WaitingHash memory nextWaitingHash =
            Partition.supplyDivergenceInterval(waitingInterval, false);

        assertEq(0, nextWaitingHash.agreePoint);
        assertEq(2, nextWaitingHash.disagreePoint);
        assertEq(INITIAL_HASH, nextWaitingHash.agreeHash);
        assertEq(INTERMEDIATE_HASH, nextWaitingHash.disagreeHash);
    }

    function test_endPartitionAgree() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                2,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        Partition.WaitingInterval memory waitingInterval =
            Partition.supplyIntermediateHash(waitingHash, INTERMEDIATE_HASH);

        Partition.Divergence memory d =
            Partition.endPartition(waitingInterval, true);

        assertEq(1, d.divergencePoint);
        assertEq(INTERMEDIATE_HASH, d.beforeHash);
        assertEq(CLAIMER_FINAL_HASH, d.afterHash);
    }

    function test_endPartitionDisagree() public {
        Partition.WaitingHash memory waitingHash =
            Partition.createPartition(
                0,
                2,
                INITIAL_HASH,
                CLAIMER_FINAL_HASH
            );

        Partition.WaitingInterval memory waitingInterval =
            Partition.supplyIntermediateHash(waitingHash, INTERMEDIATE_HASH);

        Partition.Divergence memory d =
            Partition.endPartition(waitingInterval, false);


        assertEq(0, d.divergencePoint);
        assertEq(INITIAL_HASH, d.beforeHash);
        assertEq(INTERMEDIATE_HASH, d.afterHash);
    }


    //
    // Partition proofs
    //

    function prove_createPartition(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) public {
        if (initialPoint >= finalPoint) {
            return;
        }

        if (initialPoint + 1 >= finalPoint) {
            return;
        }

        Partition.WaitingHash memory w = Partition.createPartition(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );

        assertEq(w.agreePoint, initialPoint);
        assertEq(w.disagreePoint, finalPoint);
        assertEq(w.agreeHash, initialHash);
        assertEq(w.disagreeHash, claimerFinalHash);
    }

    function proveFail_createPartition(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) public pure {
        require(initialPoint + 1 >= finalPoint);

        Partition.createPartition(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );
    }

    function prove_musts(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) public {
        if (initialPoint >= finalPoint) {
            return;
        }

        Partition.WaitingHash memory waitingHash =
            Partition.WaitingHash(
                initialPoint,
                finalPoint,
                initialHash,
                claimerFinalHash
            );

        assertTrue(
            Partition.mustEndPartition(waitingHash) &&
            !Partition.mustContinuePartition(waitingHash) ||
            !Partition.mustEndPartition(waitingHash) &&
            Partition.mustContinuePartition(waitingHash)
        );
    }

    function prove_partition(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash,
        uint8 agreeProxy,
        bytes32 intermediateHash
    ) public {
        if (initialPoint >= finalPoint) {
            return;
        }

        Partition.WaitingHash memory waitingHash =
            Partition.WaitingHash(
                initialPoint,
                finalPoint,
                initialHash,
                claimerFinalHash
            );

        if (Partition.mustEndPartition(waitingHash)) {
            return;
        }

        Partition.WaitingInterval memory waitingInterval =
            Partition.supplyIntermediateHash(waitingHash, intermediateHash);

        Partition.WaitingHash memory w = waitingInterval.waitingHash;
        compareWaitingHash(waitingHash, w);
        assertTrue(Partition.mustContinuePartition(waitingHash));

        if (agreeProxy == 0) {
            if (w.agreePoint + 2 == w.disagreePoint) {
                Partition.Divergence memory d =
                    Partition.endPartition(waitingInterval, true);

                assertEq(d.divergencePoint, w.agreePoint + 1);
                assertEq(d.beforeHash, intermediateHash);
                assertEq(d.afterHash, claimerFinalHash);
            } else {
                Partition.WaitingHash memory nextWaitingHash =
                    Partition.supplyDivergenceInterval(waitingInterval, true);

                assertEq(
                    Arithmetic.semiSum(initialPoint, finalPoint),
                    nextWaitingHash.agreePoint
                );
                assertEq(w.disagreePoint, nextWaitingHash.disagreePoint);
                assertEq(intermediateHash, nextWaitingHash.agreeHash);
                assertEq(claimerFinalHash, nextWaitingHash.disagreeHash);
            }
        } else {
            if (w.agreePoint + 2 == w.disagreePoint ||
                w.agreePoint + 3 == w.disagreePoint) {
                Partition.Divergence memory d =
                    Partition.endPartition(waitingInterval, false);

                assertEq(d.divergencePoint, w.agreePoint);
                assertEq(d.beforeHash, initialHash);
                assertEq(d.afterHash, intermediateHash);
            } else {
                Partition.WaitingHash memory nextWaitingHash =
                    Partition.supplyDivergenceInterval(waitingInterval, false);

                assertEq(w.agreePoint, nextWaitingHash.agreePoint);
                assertEq(
                    Arithmetic.semiSum(initialPoint, finalPoint),
                    nextWaitingHash.disagreePoint
                );
                assertEq(initialHash, nextWaitingHash.agreeHash);
                assertEq(intermediateHash, nextWaitingHash.disagreeHash);
            }
        }
    }


    //
    // SemiSum tests
    //

    function test_semiSum() public{
        assertEq(0, Arithmetic.semiSum(0, 0));
        assertEq(0, Arithmetic.semiSum(0, 1));
        assertEq(1, Arithmetic.semiSum(0, 2 ));
        assertEq(1, Arithmetic.semiSum(0, 3));
        assertEq(2, Arithmetic.semiSum(0, 4));
        assertEq(15, Arithmetic.semiSum(10, 20));

        assertEq(
            9223372036854775807,
            Arithmetic.semiSum(
                type(uint64).min, //0
                type(uint64).max //18446744073709551615
            )
        );
    }

    function testFail_semiSum() public pure {
        uint64 a = 1;
        uint64 b = 0;

        Arithmetic.semiSum(a, b);
    }

    function testFail_semiSumOverflow() public pure {
        uint64 a = type(uint64).max; //18446744073709551615
        uint64 b = type(uint64).min; //0

        Arithmetic.semiSum(a, b);
    }


    //
    // Internal helper methods
    //

    bytes32 constant INITIAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;
    bytes32 constant CLAIMER_FINAL_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000001;
    bytes32 constant INTERMEDIATE_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000002;

    function compareWaitingHash(
        Partition.WaitingHash memory w1,
        Partition.WaitingHash memory w2
    )
        internal
    {
        assertEq(w1.agreePoint, w2.agreePoint);
        assertEq(w1.disagreePoint, w2.disagreePoint);
        assertEq(w1.agreeHash, w2.agreeHash);
        assertEq(w1.disagreeHash, w2.disagreeHash);
    }
}
