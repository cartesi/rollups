// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/partition/PartitionEnum.sol";
import "../../src/partition/Partition.sol";

contract TestPartitionEnum is Test {
    function setUp() public {}

    //
    // Enum tests
    //

    function test_enumOfWaitingHash() public {
        Partition.WaitingHash memory waitingHash = createWaitingHash();

        PartitionEnum.T memory enumWaitingHash = PartitionEnum.T(
            PartitionEnum.Tag.WaitingHash,
            abi.encode(waitingHash)
        );

        PartitionEnum.T memory newEnumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        compareWaitingHashEnum(enumWaitingHash, newEnumWaitingHash);
    }

    function testFail_enumOfWaitingHash() public {
        Partition.WaitingHash memory waitingHash = createWaitingHash();

        PartitionEnum.T memory enumWaitingHash = PartitionEnum.T(
            PartitionEnum.Tag.WaitingInterval,
            abi.encode(waitingHash)
        );

        PartitionEnum.T memory newEnumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        compareWaitingHashEnum(enumWaitingHash, newEnumWaitingHash);
    }

    function test_enumOfWaitingInterval() public {
        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval();

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum.T(
            PartitionEnum.Tag.WaitingInterval,
            abi.encode(waitingInterval)
        );

        PartitionEnum.T memory newEnumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        compareWaitingIntervalEnum(enumWaitingInterval, newEnumWaitingInterval);
    }

    function testFail_enumOfWaitingInterval() public {
        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval();

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum.T(
            PartitionEnum.Tag.WaitingHash,
            abi.encode(waitingInterval)
        );

        PartitionEnum.T memory newEnumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        compareWaitingIntervalEnum(enumWaitingInterval, newEnumWaitingInterval);
    }

    function test_getWaitingHashVariant() public {
        Partition.WaitingHash memory waitingHash = createWaitingHash();

        PartitionEnum.T memory enumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        assertTrue(enumWaitingHash._tag == PartitionEnum.Tag.WaitingHash);

        Partition.WaitingHash memory newWaitingHash = PartitionEnum
            .getWaitingHashVariant(enumWaitingHash);

        compareWaitingHash(waitingHash, newWaitingHash);
    }

    function testFail_getWaitingHashVariant() public pure {
        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval();

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        PartitionEnum.getWaitingHashVariant(enumWaitingInterval);
    }

    function test_getWaitingIntervalVariant() public {
        Partition.WaitingInterval
            memory waitingInterval = createWaitingInterval();

        PartitionEnum.T memory enumT = PartitionEnum.enumOfWaitingInterval(
            waitingInterval
        );

        assertTrue(enumT._tag == PartitionEnum.Tag.WaitingInterval);

        Partition.WaitingInterval memory newWaitingInterval = PartitionEnum
            .getWaitingIntervalVariant(enumT);

        compareWaitingInterval(waitingInterval, newWaitingInterval);
    }

    function testFail_getWaitingIntervalVariant() public pure {
        Partition.WaitingHash memory waitingHash = createWaitingHash();
        PartitionEnum.T memory enumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        PartitionEnum.getWaitingIntervalVariant(enumWaitingHash);
    }

    //
    // Enum proofs
    //

    function prove_waitingHash(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) public {
        Partition.WaitingHash memory waitingHash = Partition.WaitingHash(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );

        PartitionEnum.T memory enumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);
        assertTrue(PartitionEnum.isWaitingHashVariant(enumWaitingHash));
        assertTrue(!PartitionEnum.isWaitingIntervalVariant(enumWaitingHash));

        Partition.WaitingHash memory newWaitingHash = PartitionEnum
            .getWaitingHashVariant(enumWaitingHash);

        compareWaitingHash(waitingHash, newWaitingHash);
    }

    function proveFail_waitingHash(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) public pure {
        Partition.WaitingHash memory waitingHash = Partition.WaitingHash(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );

        PartitionEnum.T memory enumWaitingHash = PartitionEnum
            .enumOfWaitingHash(waitingHash);

        PartitionEnum.getWaitingIntervalVariant(enumWaitingHash);
    }

    function prove_waitingInterval(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash,
        bytes32 intermediateHash
    ) public {
        Partition.WaitingInterval memory waitingInterval = Partition
            .WaitingInterval(
                Partition.WaitingHash(
                    initialPoint,
                    finalPoint,
                    initialHash,
                    claimerFinalHash
                ),
                intermediateHash
            );

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);
        assertTrue(PartitionEnum.isWaitingIntervalVariant(enumWaitingInterval));
        assertTrue(!PartitionEnum.isWaitingHashVariant(enumWaitingInterval));

        Partition.WaitingInterval memory newWaitingInterval = PartitionEnum
            .getWaitingIntervalVariant(enumWaitingInterval);

        compareWaitingInterval(waitingInterval, newWaitingInterval);
    }

    function proveFail_waitingInterval(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash,
        bytes32 intermediateHash
    ) public pure {
        Partition.WaitingInterval memory waitingInterval = Partition
            .WaitingInterval(
                Partition.WaitingHash(
                    initialPoint,
                    finalPoint,
                    initialHash,
                    claimerFinalHash
                ),
                intermediateHash
            );

        PartitionEnum.T memory enumWaitingInterval = PartitionEnum
            .enumOfWaitingInterval(waitingInterval);

        PartitionEnum.getWaitingHashVariant(enumWaitingInterval);
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

    function createWaitingHash()
        internal
        pure
        returns (Partition.WaitingHash memory)
    {
        uint64 initialPoint = 1;
        uint64 finalPoint = 4;
        bytes32 initialHash = INITIAL_HASH;
        bytes32 claimerFinalHash = CLAIMER_FINAL_HASH;

        Partition.WaitingHash memory waitingHash = Partition.createPartition(
            initialPoint,
            finalPoint,
            initialHash,
            claimerFinalHash
        );

        return waitingHash;
    }

    function createWaitingInterval()
        internal
        pure
        returns (Partition.WaitingInterval memory)
    {
        bytes32 intermediateHash = INTERMEDIATE_HASH;
        Partition.WaitingHash memory waitingHash = createWaitingHash();

        Partition.WaitingInterval memory waitingInterval = Partition
            .WaitingInterval(waitingHash, intermediateHash);

        return waitingInterval;
    }

    function compareWaitingHash(
        Partition.WaitingHash memory w1,
        Partition.WaitingHash memory w2
    ) internal {
        assertEq(w1.agreePoint, w2.agreePoint);
        assertEq(w1.disagreePoint, w2.disagreePoint);
        assertEq(w1.agreeHash, w2.agreeHash);
        assertEq(w1.disagreeHash, w2.disagreeHash);
    }

    function compareWaitingInterval(
        Partition.WaitingInterval memory w1,
        Partition.WaitingInterval memory w2
    ) internal {
        assertEq(w1.intermediateHash, w2.intermediateHash);
        compareWaitingHash(w1.waitingHash, w2.waitingHash);
    }

    function compareWaitingHashEnum(
        PartitionEnum.T memory ew1,
        PartitionEnum.T memory ew2
    ) internal {
        assertTrue(ew1._tag == ew2._tag);

        Partition.WaitingHash memory w1 = PartitionEnum.getWaitingHashVariant(
            ew1
        );

        Partition.WaitingHash memory w2 = PartitionEnum.getWaitingHashVariant(
            ew2
        );

        compareWaitingHash(w1, w2);
    }

    function compareWaitingIntervalEnum(
        PartitionEnum.T memory ew1,
        PartitionEnum.T memory ew2
    ) internal {
        assertTrue(ew1._tag == ew2._tag);

        Partition.WaitingInterval memory w1 = PartitionEnum
            .getWaitingIntervalVariant(ew1);

        Partition.WaitingInterval memory w2 = PartitionEnum
            .getWaitingIntervalVariant(ew2);

        compareWaitingInterval(w1, w2);
    }
}
