import { BytesLike } from "@ethersproject/bytes";
import epochStateV from "./voucher-proof-foundry.json";
import epochStateN from "./decoded_proof_notice.json";

// run `npx ts-node genProof.ts` to generate Solidity version of proofs from json files
// If json file needs to be updated, instructions below are similar as in the `test/OutputFacet.ts` file
// 1. uncomment `console.log` in `CartesiDApp.t.sol` file to see what the values of payload and destination should be.
// 2. we need to use the script `gen-proofs.sh` here[1]. It originally has 2 vouchers/notices. Make it into 6.
//    Replace `PAYLOAD` and `MSG_SENDER` accordingly.
//    For Apple silicon users, use long duration of `sleep` command before `# Finish epoch`. For example, `sleep 10`.
// 3. `gen-proofs.sh` outputs a JSON file with proofs in base64 encoding. This tool[2] converts base64 to hex.
//    To install: `pip install base64-to-hex-converter`
//    To run: `python -m b64to16 proof.json`
// 4. run this script to generate Solidity version of proofs
//
// ref links:
// [1]: https://github.com/cartesi-corp/machine-emulator/tree/feature/gen-proofs/tools/gen-proofs
// [2]: https://pypi.org/project/base64-to-hex-converter/
//
// Note: the script in [1] uses the same payload for both a voucher and a notice. But keep in mind that the encodings of
//       notices and vouchers are different. We use 2 notices from the original script. The notice proofs will not change.

// If no need to generate sol codes for some scenarios, comment them out

// Scenarios of vouchers:
// 0: simple_function()
// 1: simple_function(bytes32)
// 2: nonExistent()
// 3: SimpleToken.transfer(address,uint256)
// 4: ether transfer
// 5: NFT transfer
let buildSolForVouchers = [
    //0,
    //1,
    //2,
    3, 4, 5,
];

// Scenarios of notices:
// 0: "0xdeadbeef"
// 1: "0xbeefdead"
let buildSolForNotices = [0, 1];

interface OutputValidityProof {
    epochInputIndex: number;
    outputIndex: number;
    outputHashesRootHash: BytesLike;
    vouchersEpochRootHash: BytesLike;
    noticesEpochRootHash: BytesLike;
    machineStateHash: BytesLike;
    keccakInHashesSiblings: BytesLike[];
    outputHashesInEpochSiblings: BytesLike[];
}

function setupVoucherProof(epochInputIndex: number): OutputValidityProof {
    let voucherDataKeccakInHashes =
        epochStateV.processedInputs[epochInputIndex].acceptedData.vouchers[0]
            .keccakInVoucherHashes;
    let voucherHashesInEpoch =
        epochStateV.processedInputs[epochInputIndex].voucherHashesInEpoch
            .siblingHashes;
    var siblingKeccakInHashesV: BytesLike[] = [];
    var voucherHashesInEpochSiblings: BytesLike[] = [];
    voucherDataKeccakInHashes.siblingHashes.forEach((element: any) => {
        siblingKeccakInHashesV.push(element.data);
    });
    voucherHashesInEpoch.forEach((element: any) => {
        voucherHashesInEpochSiblings.push(element.data);
    });
    let voucherProof: OutputValidityProof = {
        epochInputIndex: epochInputIndex,
        outputIndex: 0,
        outputHashesRootHash: voucherDataKeccakInHashes.rootHash.data,
        vouchersEpochRootHash: epochStateV.mostRecentVouchersEpochRootHash.data,
        noticesEpochRootHash: epochStateV.mostRecentNoticesEpochRootHash.data,
        machineStateHash: epochStateV.mostRecentMachineHash.data,
        keccakInHashesSiblings: siblingKeccakInHashesV.reverse(),
        outputHashesInEpochSiblings: voucherHashesInEpochSiblings.reverse(),
    };
    return voucherProof;
}

function setupNoticeProof(epochInputIndex: number): OutputValidityProof {
    let noticeDataKeccakInHashes =
        epochStateN.processedInputs[epochInputIndex].acceptedData.notices[0]
            .keccakInNoticeHashes;
    let noticeHashesInEpoch =
        epochStateN.processedInputs[epochInputIndex].noticeHashesInEpoch
            .siblingHashes;
    var siblingKeccakInHashesN: BytesLike[] = [];
    var noticeHashesInEpochSiblingsN: BytesLike[] = [];
    noticeDataKeccakInHashes.siblingHashes.forEach((element) => {
        siblingKeccakInHashesN.push(element.data);
    });
    noticeHashesInEpoch.forEach((element) => {
        noticeHashesInEpochSiblingsN.push(element.data);
    });
    let noticeProof: OutputValidityProof = {
        epochInputIndex: epochInputIndex,
        outputIndex: 0,
        outputHashesRootHash: noticeDataKeccakInHashes.rootHash.data,
        vouchersEpochRootHash: epochStateN.mostRecentVouchersEpochRootHash.data,
        noticesEpochRootHash: epochStateN.mostRecentNoticesEpochRootHash.data,
        machineStateHash: epochStateN.mostRecentMachineHash.data,
        keccakInHashesSiblings: siblingKeccakInHashesN.reverse(), // from top-down to bottom-up
        outputHashesInEpochSiblings: noticeHashesInEpochSiblingsN.reverse(),
    };
    return noticeProof;
}

function buildSolCodes(p: OutputValidityProof): string {
    return (
        '\
    // SPDX-License-Identifier: UNLICENSED\n\
    pragma solidity ^0.8.13;\n\n\
    // THIS FILE WAS AUTOMATICALLY GENERATED BY `genProof.ts`.\n\n\
    import {OutputValidityProof} from "contracts/library/LibOutputValidation.sol";\n\n\
    contract Proof {\n\
        OutputValidityProof public proof;\n\n\
        constructor() {\n\
            proof.epochInputIndex = ' +
        p.epochInputIndex +
        ";\n\
            proof.outputIndex = " +
        p.outputIndex +
        ";\n\
            proof.outputHashesRootHash = " +
        p.outputHashesRootHash +
        ";\n\
            proof.vouchersEpochRootHash = " +
        p.vouchersEpochRootHash +
        ";\n\
            proof.noticesEpochRootHash = " +
        p.noticesEpochRootHash +
        ";\n\
            proof.machineStateHash = " +
        p.machineStateHash +
        ";\n\
            uint256[" +
        p.keccakInHashesSiblings.length +
        "] memory array1 = [\n\
                " +
        p.keccakInHashesSiblings +
        "\n\
            ];\n\
            for (uint256 i; i < array1.length; ++i) {\n\
                proof.keccakInHashesSiblings.push(bytes32(array1[i]));\n\
            }\n\
            uint256[" +
        p.outputHashesInEpochSiblings.length +
        "] memory array2 = [\n\
                " +
        p.outputHashesInEpochSiblings +
        "\n\
            ];\n\
            for (uint256 i; i < array2.length; ++i) {\n\
                proof.outputHashesInEpochSiblings.push(bytes32(array2[i]));\n\
            }\n\
        }\n\n\
        function getArray1() public view returns (bytes32[] memory) {\n\
            return proof.keccakInHashesSiblings;\n\
        }\n\n\
        function getArray2() public view returns (bytes32[] memory) {\n\
            return proof.outputHashesInEpochSiblings;\n\
        }\n\
    }\n\
    "
    );
}

// generate sol codes for vouchers
for (let vIndex of buildSolForVouchers) {
    const fs = require("fs");

    let p = setupVoucherProof(vIndex);

    let solidityCode = buildSolCodes(p);

    // write to file
    fs.writeFile("voucherProof" + vIndex + ".sol", solidityCode, (err: any) => {
        // throws an error, you could also catch it here
        if (err) throw err;

        // success case, the file was saved
        console.log("voucher proof " + vIndex + " generated!");
    });
}

// generate sol codes for notices
for (let nIndex of buildSolForNotices) {
    const fs = require("fs");

    let p = setupNoticeProof(nIndex);

    let solidityCode = buildSolCodes(p);

    // write to file
    fs.writeFile("noticeProof" + nIndex + ".sol", solidityCode, (err: any) => {
        // throws an error, you could also catch it here
        if (err) throw err;

        // success case, the file was saved
        console.log("notice proof " + nIndex + " generated!");
    });
}
