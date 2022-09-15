// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "utils/Arithmetic.sol";

library Partition {
    struct WaitingHash {
        uint64 agreePoint;
        uint64 disagreePoint;

        bytes32 agreeHash;
        bytes32 disagreeHash;
    }

    struct WaitingInterval {
        WaitingHash waitingHash;
        bytes32 intermediateHash;
    }

    struct Divergence {
        uint64 divergencePoint;
        bytes32 beforeHash;
        bytes32 afterHash;
    }


    //
    // Methods
    //

    function createPartition(
        uint64 initialPoint,
        uint64 finalPoint,
        bytes32 initialHash,
        bytes32 claimerFinalHash
    ) external pure returns(WaitingHash memory) {
        require(
            initialPoint < finalPoint,
            "initial point ahead of final point"
        );

        require(
            initialPoint + 1 < finalPoint,
            "partition already finished"
        );

        return WaitingHash(
          initialPoint,
          finalPoint,
          initialHash,
          claimerFinalHash
        );
    }

    function supplyIntermediateHash(
        WaitingHash memory waitingHash,
        bytes32 intermediateHash
    ) external pure returns(WaitingInterval memory) {
        return WaitingInterval(waitingHash, intermediateHash);
    }

    function supplyDivergenceInterval(
        WaitingInterval memory waitingInterval,
        bool agree
    ) external pure returns(WaitingHash memory) {
        WaitingHash memory waitingHash =
            advancePartition(waitingInterval, agree);

        require(mustContinuePartition(waitingHash), "must end partition");

        return waitingHash;
    }

    function endPartition(
        WaitingInterval memory waitingInterval,
        bool agree
    ) external pure returns(Divergence memory) {
        WaitingHash memory waitingHash =
            advancePartition(waitingInterval, agree);

        require(mustEndPartition(waitingHash), "can't end partition yet");

        return Divergence(
            waitingHash.agreePoint,
            waitingHash.agreeHash,
            waitingHash.disagreeHash
        );
    }

    function mustEndPartition(
        WaitingHash memory waitingHash
    ) public pure returns(bool) {
        return waitingHash.agreePoint + 1 == waitingHash.disagreePoint;
    }

    function mustContinuePartition(
        WaitingHash memory waitingHash
    ) public pure returns(bool) {
        return waitingHash.agreePoint + 1 < waitingHash.disagreePoint;
    }



    //
    // Internal
    //

    function advancePartition(
        WaitingInterval memory waitingInterval,
        bool agree
    ) private pure returns (WaitingHash memory) {
        WaitingHash memory waitingHash = waitingInterval.waitingHash;

        bytes32 newHash = waitingInterval.intermediateHash;
        uint64 newPoint = Arithmetic.semiSum(
            waitingHash.agreePoint,
            waitingHash.disagreePoint
        );

        if (agree) {
            return WaitingHash(
                newPoint,
                waitingHash.disagreePoint,
                newHash,
                waitingHash.disagreeHash
            );
        } else {
            return WaitingHash(
                waitingHash.agreePoint,
                newPoint,
                waitingHash.agreeHash,
                newHash
            );
        }
    }
}
