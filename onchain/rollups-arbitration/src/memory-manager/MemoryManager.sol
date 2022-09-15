// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "utils/Memory.sol";
import { Word } from "utils/Word.sol";
import { Merkle } from "utils/Merkle.sol";
import { AccessProof } from "./AccessProof.sol";

library MemoryManager {
    using Merkle for bytes32[];
    using Word for Word.Slot;

    using AccessProof for AccessProof.RollingHash;
    using AccessProof for AccessProof.Bit256Array;

    struct Context {
        // Current rolling hash of all read/write proved
        AccessProof.RollingHash memoryAccessRollingHash;

        // Machine Merkle hash before write operations
        Merkle.Hash initialMachineHash;

        // Hash after write operations have been proved
        Merkle.Hash currentMachineHash;
    }

    modifier validWord(
        Context memory context,
        bytes32[] calldata proof,
        Word.Slot calldata slot
    ) {
        require(
            proof.isValidMachine(context.currentMachineHash, slot),
            "Merkle proof does not match"
        );

        _;
    }


    //
    // Methods
    //

    function newMemoryManager(
        Merkle.Hash initialMachineHash
    ) external pure returns(Context memory) {
        return Context(
            AccessProof.initialRollingHash(),
            initialMachineHash,
            initialMachineHash
        );
    }

    function addRead(
        Context memory context,
        bytes32[] calldata proof,
        Word.Slot calldata slot
    )
        external
        pure
        validWord(context, proof, slot)
        returns(Context memory)
    {
        AccessProof.RollingHash newRollingHash = AccessProof.nextRollingHash(
            context.memoryAccessRollingHash,
            slot,
            true
        );

        return Context(
            newRollingHash,
            context.initialMachineHash,
            context.currentMachineHash
        );
    }

    function addWrite(
        Context memory context,
        bytes32[] calldata proof,
        Word.Slot calldata slot,
        Word.Value newValue
    )
        external
        pure
        validWord(context, proof, slot)
        returns(Context memory)
    {
        // TODO review this
        AccessProof.RollingHash newRollingHash = AccessProof.nextRollingHash(
            context.memoryAccessRollingHash,
            slot,
            false
        );

        Merkle.Hash newHash = proof.replaceWordInMachine(
            slot.updateValue(newValue)
        );

        return Context(
            newRollingHash,
            context.initialMachineHash,
            newHash
        );
    }

    function proveAccess(
        AccessProof.RollingHash finalRollingHash,
        Word.Slot[] calldata slots,
        AccessProof.Bit256Array isReads
    ) external pure returns(bool) {
        AccessProof.RollingHash currentRollingHash =
            AccessProof.initialRollingHash();

        for (uint i = 0; i < slots.length; i++) {
            currentRollingHash = AccessProof.nextRollingHash(
                currentRollingHash,
                slots[i],
                isReads.isOne(i)
            );
        }

        return currentRollingHash.eq(finalRollingHash);
    }
}
