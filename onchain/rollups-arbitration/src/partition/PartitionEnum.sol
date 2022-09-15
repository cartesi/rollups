// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./Partition.sol";

library PartitionEnum {
    /*
    let type PartitionEnum.T =
        | WaitingHash of Partition.WaitingHash
        | WaitingInterval of Partition.WaitingInterval
     */

    enum Tag {WaitingHash, WaitingInterval}

    struct T {
        Tag _tag;
        bytes _data;
    }

    function enumOfWaitingHash(
        Partition.WaitingHash memory waitingHash
    ) external pure returns (T memory) {
        return T(Tag.WaitingHash, abi.encode(waitingHash));
    }

    function enumOfWaitingInterval(
        Partition.WaitingInterval memory _reply
    ) external pure returns (T memory) {
        return T(Tag.WaitingInterval, abi.encode(_reply));
    }

    function isWaitingHashVariant(T memory t) external pure returns (bool) {
        return t._tag == Tag.WaitingHash;
    }

    function isWaitingIntervalVariant(T memory t) external pure returns (bool) {
        return t._tag == Tag.WaitingInterval;
    }

    function getWaitingHashVariant(
        T memory t
    ) external pure returns (Partition.WaitingHash memory) {
        require(t._tag == Tag.WaitingHash);
        return abi.decode(t._data, (Partition.WaitingHash));
    }

    function getWaitingIntervalVariant(
        T memory t
    ) external pure returns (Partition.WaitingInterval memory) {
        require(t._tag == Tag.WaitingInterval);
        return abi.decode(t._data, (Partition.WaitingInterval));
    }
}
