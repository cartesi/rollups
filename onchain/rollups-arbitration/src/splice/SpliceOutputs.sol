// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Word } from "utils/Word.sol";

import "../partition/Partition.sol";
import "../epoch-hash-split/EpochHashSplit.sol";
import "./SpliceDataSource.sol";
import "./SpliceUtils.sol";

library SpliceOutputs {
    using Merkle for bytes32[];

    struct SpliceOutputsProofs {
        Merkle.Hash outputLeafMerkleHash;
        bytes32[] outputsMemoryRangeProof;
        bytes32[] outputsLeafProof;
    }

    function spliceOutputs(
        Merkle.Hash rootMachineHashAfter,
        Merkle.Hash rootOutputsHashBefore,
        SpliceOutputsProofs calldata proofs,
        SpliceDataSource.AddressSpace memory addressSpace,
        uint64 inputIndex
    )
        external
        pure
        returns(Merkle.Hash)
    {
        require(
            proofs.outputsLeafProof.isValidOutputs(
                rootOutputsHashBefore,
                inputIndex,
                Merkle.zeroHash()
            ),
            "Outputs merkle proof does not match"
        );

        require(
            proofs.outputsMemoryRangeProof.isValidMachine(
                rootMachineHashAfter,
                addressSpace.outputHashesAddress,
                Merkle.Node(
                    proofs.outputLeafMerkleHash,
                    addressSpace.outputHashesLog2Size
                )
            ),
            "Machine merkle proof does not match"
        );

        return proofs.outputsLeafProof.replaceLeafInOutputs(
            inputIndex,
            proofs.outputLeafMerkleHash
        );
    }
}
