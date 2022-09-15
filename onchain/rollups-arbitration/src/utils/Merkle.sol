// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "./Memory.sol";
import { Word } from "./Word.sol";

import { Merkle as MerkleLib } from "@cartesi/util/contracts/Merkle.sol";

library Merkle {

    //
    // Merkle Hash
    //

    type Hash is bytes32;

    using Merkle for Hash;

    function unwrap(Hash h) internal pure returns (bytes32) {
        return Hash.unwrap(h);
    }

    function zeroHash() internal pure returns (Hash) {
        return Hash.wrap(bytes32(0x0));
    }

    function eq(Hash h1, Hash h2) internal pure returns (bool) {
        return h1.unwrap() == h2.unwrap();
    }

    function concat(Hash h1, Hash h2) internal pure returns (bytes memory) {
        return abi.encode(h1.unwrap(), h2.unwrap());
    }

    function concatAndHash(Hash h1, Hash h2) public pure returns (bytes32) {
        return keccak256(h1.concat(h2));
    }

    //
    // Merkle node
    //

    struct Node {
        Hash hash;
        Memory.Log2Size log2Size;
    }

    using Merkle for Node;

    function merkleize(
        bytes calldata content,
        Memory.Log2Size log2Size
    )
        internal
        pure
        returns (Node memory)
    {
        bytes32 hash = MerkleLib.getMerkleRootFromBytes(
            content,
            Memory.Log2Size.unwrap(log2Size)
        );

        return Node(
            Hash.wrap(hash),
            log2Size
        );
    }

    function merkleOfZeroes(
        Memory.Log2Size log2Size
    )
        internal
        pure
        returns (Node memory)
    {
        bytes32 hash = MerkleLib.getMerkleRootFromBytes(
            "",
            Memory.Log2Size.unwrap(log2Size)
        );

        return Node(
            Hash.wrap(hash),
            log2Size
        );
    }

    function nodeWithSize(
        Hash hash,
        Memory.Log2Size log2Size
    )
        internal
        pure
        returns (Node memory)
    {
        return Node(hash, log2Size);
    }

    function updateHash(Node memory node, Hash newHash)
        internal pure returns (Node memory)
    {
        return Node(newHash, node.log2Size);
    }



    //
    // Merkle word value
    //

    function merkleHash(Word.Value v) internal pure returns (Hash) {
        return Hash.wrap(v.hash());
    }

    function merkleNodeOfWord(Word.Value v)  internal pure returns (Node memory) {
        return Node(
            v.merkleHash(),
            Memory.wordLog2Size()
        );
    }


    //
    // Merkle replacements
    //

    using Word for Word.Value;
    using Merkle for Word.Value;
    using Merkle for bytes32[];
    using Memory for Memory.Log2Size;

    function isValidMachine(
        bytes32[] calldata merkleProof,
        Merkle.Hash rootHash,
        Memory.Address contentAddress,
        Merkle.Node memory contentNode
    )
        internal
        pure
        returns(bool)
    {
        return merkleProof
            .replaceMemoryRangeInMachine(
                contentAddress,
                contentNode
            )
            .eq(rootHash);
    }

    function isValidMachine(
        bytes32[] calldata merkleProof,
        Merkle.Hash rootHash,
        Word.Slot memory word
    )
        internal
        pure
        returns(bool)
    {
        return merkleProof
            .replaceWordInMachine(word)
            .eq(rootHash);
    }

    function isValidOutputs(
        bytes32[] calldata merkleProof,
        Merkle.Hash rootHash,
        uint64 leafIndex,
        Hash leafContent
    )
        internal
        pure
        returns(bool)
    {
        return merkleProof
            .replaceLeafInOutputs(
                leafIndex,
                leafContent
            )
            .eq(rootHash);
    }

    function replaceWordInMachine(
        bytes32[] calldata proof,
        Word.Slot memory word
    )
        internal
        pure
        returns (Merkle.Hash)
    {
        return replaceMemoryRangeInMachine(
            proof,
            word.memoryAddress,
            word.value.merkleNodeOfWord()
        );
    }

    function replaceMemoryRangeInMachine(
        bytes32[] calldata merkleProof,
        Memory.Address nodeAddress,
        Merkle.Node memory replacementNode
    )
        internal
        pure
        returns(Merkle.Hash)
    {
        return Merkle.Hash.wrap(
            MerkleLib.getRootAfterReplacementInDrive(
                Memory.Address.unwrap(nodeAddress),
                Memory.Log2Size.unwrap(replacementNode.log2Size),
                Memory.machineLog2Size().uint64_of_size(),
                Hash.unwrap(replacementNode.hash),
                merkleProof
            )
        );
    }

    function replaceLeafInOutputs(
        bytes32[] calldata merkleProof,
        uint64 leafIndex,
        Hash leafHash
    )
        internal
        pure
        returns(Merkle.Hash)
    {
        return Merkle.Hash.wrap(
            MerkleLib.getRootAfterReplacementInDrive(
                leafIndex,
                Memory.hashLog2Size().uint64_of_size(),
                Memory.outputsLog2Size().uint64_of_size(),
                leafHash.unwrap(),
                merkleProof
            )
        );
    }
}
