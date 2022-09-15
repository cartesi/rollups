// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./TwoPartyArbitration.sol";

contract TwoPartyArbitrationInstantiator {
    using TwoPartyArbitrationEnum for TwoPartyArbitrationEnum.T;
    using TwoPartyArbitration for TwoPartyArbitration.Context;

    enum Status {Uninitialized, Ongoing, ChallengerWon, ClaimerWon}

    mapping(uint256 => bool) statusMapping;
    mapping(uint256 => bytes32) contextHashes; //TODO: (doubt) when arbitration finishes, we delete the contextHash entry so we loose the reference with the stateMapping
    uint256 currentIndex; 

    event TwoPartyArbitrationNewContext(
        uint256 indexed index,
        bytes context
    );

    event ArbitrationFinishedChallengerWon(
        uint256 indexed index
    );

    event ArbitrationFinishedClaimerWon(
        uint256 indexed index
    );

    modifier validIndexAndProof(uint256 index, bytes calldata proof) {
        require(
            index < currentIndex,
            "supplied index higher than current index"
        );

        bytes32 ctxHash = contextHashes[index];

        require(
            ctxHash != bytes32(0x0),
            "arbitration is done"
        );

        require(
            keccak256(proof) == ctxHash,
            "context proof does not match"
        );

        _;
    }

    function arbitrationStatus(uint256 index) external view returns(Status) {
        if (index >= currentIndex) {
            return Status.Uninitialized;
        } else if (contextHashes[index] != bytes32(0x0)) {
            return Status.Ongoing;
        } else {
            return statusMapping[index] ?
                Status.ChallengerWon : Status.ClaimerWon; //TODO: (doubt) if there is no statusmapping on arbitration for a index, the default is claimerwon?
        }
    }

    function createArbitration(
        TwoPartyArbitration.ArbitrationArguments memory arguments
    )
        external
        returns(uint256)
    {
        TwoPartyArbitration.Context memory ctx =
            TwoPartyArbitration.createArbitration(
                arguments
            );

        saveContext(currentIndex, ctx);
        return currentIndex++;
    }


    //
    // Timeout methods
    //

    function challengerWinByTimeout(
        uint256 index,
        bytes calldata proof
    )
        external
        validIndexAndProof(index, proof)
    {
        getCtx(proof).challengerWinByTimeout();
        challengerWins(index);
    }

    function claimerWinByTimeout(
        uint256 index,
        bytes calldata proof
    )
        external
        validIndexAndProof(index, proof)
    {
        getCtx(proof).claimerWinByTimeout();
        claimerWins(index);
    }



    //
    // Advance State Partition
    //

    function stateAdvanceSupplyIntermediateHash(
        uint256 index,
        bytes calldata proof,
        bytes32 replyHash
    )
        external
        validIndexAndProof(index, proof)
    {
        TwoPartyArbitration.Context memory newContext = getCtx(proof)
            .stateAdvanceSupplyIntermediateHash(replyHash);

        saveContext(index, newContext);
    }

    function stateAdvanceSupplyDivergenceInterval(
        uint256 index,
        bytes calldata proof,
        bool agree
    )
        external
        validIndexAndProof(index, proof)
    {
        TwoPartyArbitration.Context memory newContext = getCtx(proof)
            .stateAdvanceSupplyDivergenceInterval(agree);

        saveContext(index, newContext);
    }

    function stateAdvanceEndPartition(
        uint256 index,
        bytes calldata proof,
        bool agree,
        Merkle.Hash preAdvanceMachine,
        Merkle.Hash preAdvanceOutputs
    )
        external
        validIndexAndProof(index, proof)
    {
        TwoPartyArbitration.Context memory newContext = getCtx(proof)
            .stateAdvanceEndPartition(
                agree,
                preAdvanceMachine,
                preAdvanceOutputs
            );

        saveContext(index, newContext);
    }

    //
    // Epoch Hash methods //TODO
    //

    
    //
    // Internals
    //

    function saveContext(
        uint256 index,
        TwoPartyArbitration.Context memory ctx
    )
        private
    {
        bytes memory data = abi.encode(ctx);
        emit TwoPartyArbitrationNewContext(index, data);
        contextHashes[index] = keccak256(data);
    }

    function challengerWins(
        uint256 index
    )
        private
    {
        delete contextHashes[index];
        statusMapping[index] = true;
        emit ArbitrationFinishedChallengerWon(index);
    }

    function claimerWins(
        uint256 index
    )
        private
    {
        delete contextHashes[index];
        emit ArbitrationFinishedClaimerWon(index);
    }

    function getCtx(
        bytes calldata proof
    )
        private
        pure
        returns(TwoPartyArbitration.Context memory)
    {
        return abi.decode(proof, (TwoPartyArbitration.Context));
    }
}
