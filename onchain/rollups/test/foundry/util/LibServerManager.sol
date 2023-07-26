// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {Vm} from "forge-std/Vm.sol";

library LibServerManager {
    using LibServerManager for string;
    using LibServerManager for bytes32;
    using LibServerManager for RawHash;
    using LibServerManager for RawHash[];
    using LibServerManager for RawOutputValidityProof;
    using LibServerManager for RawProof;
    using LibServerManager for RawProof[];

    error InvalidOutputEnum(string);

    struct RawHash {
        bytes32 data;
    }

    struct RawOutputValidityProof {
        string inputIndexWithinEpoch;
        RawHash machineStateHash;
        RawHash noticesEpochRootHash;
        RawHash[] outputHashInOutputHashesSiblings;
        RawHash[] outputHashesInEpochSiblings;
        RawHash outputHashesRootHash;
        string outputIndexWithinInput;
        RawHash vouchersEpochRootHash;
    }

    struct RawProof {
        bytes32 context;
        string inputIndex;
        string outputEnum;
        string outputIndex;
        RawOutputValidityProof validity;
    }

    struct RawFinishEpochResponse {
        RawHash machineHash;
        RawHash noticesEpochRootHash;
        RawProof[] proofs;
        RawHash vouchersEpochRootHash;
    }

    function toUint(string memory s, Vm vm) internal pure returns (uint256) {
        return vm.parseUint(s);
    }

    function fmt(RawHash memory h) internal pure returns (bytes32) {
        return h.data;
    }

    function fmt(RawHash[] memory hs) internal pure returns (bytes32[] memory) {
        bytes32[] memory b32s = new bytes32[](hs.length);
        for (uint256 i; i < hs.length; ++i) {
            b32s[i] = hs[i].fmt();
        }
        return b32s;
    }

    function fmt(
        RawOutputValidityProof memory v,
        Vm vm
    ) internal pure returns (OutputValidityProof memory) {
        return
            OutputValidityProof({
                inputIndexWithinEpoch: v.inputIndexWithinEpoch.toUint(vm),
                outputIndexWithinInput: v.outputIndexWithinInput.toUint(vm),
                outputHashesRootHash: v.outputHashesRootHash.fmt(),
                vouchersEpochRootHash: v.vouchersEpochRootHash.fmt(),
                noticesEpochRootHash: v.noticesEpochRootHash.fmt(),
                machineStateHash: v.machineStateHash.fmt(),
                outputHashInOutputHashesSiblings: v
                    .outputHashInOutputHashesSiblings
                    .fmt(),
                outputHashesInEpochSiblings: v.outputHashesInEpochSiblings.fmt()
            });
    }

    function hash(string memory s) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(s));
    }

    function toOutputEnum(string memory s) internal pure returns (OutputEnum) {
        bytes32 h = s.hash();
        if (h == hash("VOUCHER")) {
            return OutputEnum.VOUCHER;
        } else if (h == hash("NOTICE")) {
            return OutputEnum.NOTICE;
        } else {
            revert InvalidOutputEnum(s);
        }
    }

    function toBytes(bytes32 b) internal pure returns (bytes memory) {
        return abi.encodePacked(b);
    }

    function fmt(
        RawProof memory p,
        Vm vm
    ) internal pure returns (Proof memory) {
        return
            Proof({
                inputIndex: p.inputIndex.toUint(vm),
                outputIndex: p.outputIndex.toUint(vm),
                outputEnum: p.outputEnum.toOutputEnum(),
                validity: p.validity.fmt(vm),
                context: p.context.toBytes()
            });
    }

    function fmt(
        RawProof[] memory rawps,
        Vm vm
    ) internal pure returns (Proof[] memory) {
        uint256 n = rawps.length;
        Proof[] memory ps = new Proof[](n);
        for (uint256 i; i < n; ++i) {
            ps[i] = rawps[i].fmt(vm);
        }
        return ps;
    }

    function fmt(
        RawFinishEpochResponse memory r,
        Vm vm
    ) internal pure returns (FinishEpochResponse memory) {
        return
            FinishEpochResponse({
                machineHash: r.machineHash.fmt(),
                vouchersEpochRootHash: r.vouchersEpochRootHash.fmt(),
                noticesEpochRootHash: r.noticesEpochRootHash.fmt(),
                proofs: r.proofs.fmt(vm)
            });
    }

    struct OutputValidityProof {
        uint256 inputIndexWithinEpoch;
        uint256 outputIndexWithinInput;
        bytes32 outputHashesRootHash;
        bytes32 vouchersEpochRootHash;
        bytes32 noticesEpochRootHash;
        bytes32 machineStateHash;
        bytes32[] outputHashInOutputHashesSiblings;
        bytes32[] outputHashesInEpochSiblings;
    }

    enum OutputEnum {
        VOUCHER,
        NOTICE
    }

    struct Proof {
        uint256 inputIndex;
        uint256 outputIndex;
        OutputEnum outputEnum;
        OutputValidityProof validity;
        bytes context;
    }

    struct FinishEpochResponse {
        bytes32 machineHash;
        bytes32 vouchersEpochRootHash;
        bytes32 noticesEpochRootHash;
        Proof[] proofs;
    }

    function proves(
        Proof memory p,
        OutputEnum outputEnum,
        uint256 inputIndex,
        uint256 outputIndex
    ) internal pure returns (bool) {
        return
            p.outputEnum == outputEnum &&
            p.inputIndex == inputIndex &&
            p.outputIndex == outputIndex;
    }
}
