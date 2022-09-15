// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "utils/Memory.sol";
import { Word } from "utils/Word.sol";
import { Merkle } from "utils/Merkle.sol";

import "./SpliceDataSource.sol";

library SpliceUtils {
    using Merkle for bytes32[];
    using Merkle for Merkle.Hash;
    using Merkle for Merkle.Node;
    using Word for Word.Slot;
    using Word for Word.Value;

    Word.Value constant IFLAGS_RESET_MASK = Word.Value.wrap(0x0); // TODO

    struct SpliceMachineData {
        uint64 epochIndex;
        uint64 divergenceInputIndex;
        Merkle.Hash preSpliceMachineHash;
    }

    struct SpliceMachineProofs {
        bytes inputMetadata;    //Content of input metadata 
        bytes input;            //dapp specific payload
        Merkle.Hash previousRxMerkleHash;
        Merkle.Hash previousInputMetadataMerkleHash;
        Merkle.Hash previousOutputMerkleHash;
        Word.Value previousIflags;
        bytes32[] rxProof;
        bytes32[] metadataProof;
        bytes32[] outputProof;
        bytes32[] iflagsProof;
    }

    modifier validMemoryRange(
        Merkle.Hash rootHash,
        bytes32[] calldata merkleProof,
        Memory.Address contentAddress,
        Merkle.Node memory contentNode
    ) {
        require(
            merkleProof.isValidMachine(rootHash, contentAddress, contentNode),
            "Merkle proof does not match"
        );

        _;
    }

    modifier validWord(
        Merkle.Hash rootHash,
        bytes32[] calldata merkleProof,
        Word.Slot memory slot
    ) {
        require(
            merkleProof.isValidMachine(rootHash, slot),
            "Merkle proof does not match"
        );

        _;
    }

    modifier validInput(
        bytes32 hash,
        bytes calldata inputMetadata,
        bytes calldata input
    ) {
        bytes32 metadataHash = keccak256(inputMetadata);
        bytes32 inputHash = keccak256(input);

        require(
            keccak256(abi.encode(metadataHash, inputHash)) == hash,
            "Supplied input and metadata incorrect"
        );

        _;
    }

    function spliceMachine(
        Merkle.Hash rootHash,
        bytes32 inputHash,
        SpliceMachineProofs calldata proofs,
        SpliceDataSource.AddressSpace memory addressSpace
    )
        internal
        pure
        validInput(
            inputHash,
            proofs.inputMetadata,
            proofs.input
        )
        returns(Merkle.Hash)
    {
        // The splice machine consists of four consecutive memory changes:
        //
        // 1- Change rx-buffer memory range for next input, replacing merkle tree node
        // representing the memory range by a merkle-ized next input.
        //
        // 2- Change input metadata memory range for next metadata, replacing merkle
        // tree node representing the memory range by a merkle-ized next metadata.
        //
        // 3- Zero output hashes memory range, replacing merkle tree node representing
        // the memory range by a "zero merkle tree".
        //
        // 4- Reset iflags, replacing word by reset word.


        // Replace rxBuffer with next input.
        rootHash = spliceWithContent(
            rootHash,
            proofs.rxProof,
            addressSpace.rxBufferAddress,
            Merkle.merkleize(
                proofs.input,
                addressSpace.rxBufferLog2Size
            ),
            proofs.previousRxMerkleHash
        );

        // Replace input metadata with next input.
        rootHash = spliceWithContent(
            rootHash,
            proofs.metadataProof,
            addressSpace.inputMetadataAddress,
            Merkle.merkleize(
                proofs.inputMetadata,
                addressSpace.inputMetadataLog2Size
            ),
            proofs.previousInputMetadataMerkleHash
        );


        // Replace output buffer with zeroes.
        rootHash = spliceWithZero(
            rootHash,
            proofs.outputProof,
            addressSpace.outputHashesAddress,
            Merkle.Node(
                proofs.previousOutputMerkleHash,
                addressSpace.outputHashesLog2Size
            )
        );

        // Replace iflags with reset value
        rootHash = spliceWord(
            rootHash,
            proofs.iflagsProof,
            Word.Slot(
                proofs.previousIflags,
                addressSpace.iflagsAddress
            ),
            proofs.previousIflags.or(IFLAGS_RESET_MASK)
        );

        return rootHash;
    }


    function spliceWithContent(
        Merkle.Hash rootHash,
        bytes32[] calldata merkleProof,
        Memory.Address bufferAddress,
        Merkle.Node memory newBufferNode,
        Merkle.Hash previousBufferHash
    )
        internal
        pure
        validMemoryRange(
            rootHash,
            merkleProof,
            bufferAddress,
            previousBufferHash.nodeWithSize(newBufferNode.log2Size)
        )
        returns(Merkle.Hash)
    {
        return merkleProof.replaceMemoryRangeInMachine(
            bufferAddress,
            newBufferNode
        );
    }

    function spliceWithZero(
        Merkle.Hash rootHash,
        bytes32[] calldata merkleProof,
        Memory.Address bufferAddress,
        Merkle.Node memory previousBufferNode
    )
        internal
        pure
        validMemoryRange(
            rootHash,
            merkleProof,
            bufferAddress,
            previousBufferNode
        )
        returns(Merkle.Hash)
    {
        return merkleProof.replaceMemoryRangeInMachine(
            bufferAddress,
            Merkle.merkleOfZeroes(previousBufferNode.log2Size)
        );
    }

    function spliceWord(
        Merkle.Hash rootHash,
        bytes32[] calldata merkleProof,
        Word.Slot memory slot,
        Word.Value newValue
    )
        internal
        pure
        validWord(rootHash, merkleProof, slot)
        returns(Merkle.Hash)
    {
        return merkleProof.replaceWordInMachine(
            slot.updateValue(newValue)
        );
    }
}
