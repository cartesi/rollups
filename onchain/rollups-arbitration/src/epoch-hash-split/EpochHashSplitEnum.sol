// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./EpochHashSplit.sol";

library EpochHashSplitEnum {
    /*
    let type EpochHashSplitEnum.T =
        | WaitingSubhashes of EpochHashSplit.WaitingSubhashes
        | WaitingDivergence of EpochHashSplit.WaitingDivergence
     */

    enum Tag {WaitingSubhashes, WaitingDivergence}

    struct T {
        Tag _tag;
        bytes _data;
    }

    function enumOfWaitingSubhashes(
        EpochHashSplit.WaitingSubhashes memory waitingClaim
    ) external pure returns (T memory) {
        return T(Tag.WaitingSubhashes, abi.encode(waitingClaim));
    }

    function enumOfWaitingDivergence(
        EpochHashSplit.WaitingDivergence memory claim
    ) external pure returns (T memory) {
        return T(Tag.WaitingDivergence, abi.encode(claim));
    }

    function isWaitingSubhashesVariant(T memory t) external pure returns (bool) {
        return t._tag == Tag.WaitingSubhashes;
    }

    function isWaitingDivergenceVariant(T memory t) external pure returns (bool) {
        return t._tag == Tag.WaitingDivergence;
    }

    function getWaitingSubhashesVariant(
        T memory t
    ) external pure returns (EpochHashSplit.WaitingSubhashes memory) {
        require(t._tag == Tag.WaitingSubhashes);
        return abi.decode(t._data, (EpochHashSplit.WaitingSubhashes));
    }

    function getWaitingDivergenceVariant(
        T memory t
    ) external pure returns (EpochHashSplit.WaitingDivergence memory) {
        require(t._tag == Tag.WaitingDivergence);
        return abi.decode(t._data, (EpochHashSplit.WaitingDivergence));
    }
}
