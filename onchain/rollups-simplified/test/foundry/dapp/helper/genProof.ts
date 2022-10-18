import { BytesLike } from "@ethersproject/bytes";
import epochStateV from "./voucherProofs.json";
import epochStateN from "./noticeProofs.json";

// If json file needs to be updated, instructions below are similar as in the `test/OutputFacet.ts` file
// 1. set the boolean state variable `logVouchers` to `true` in `CartesiDApp.t.sol` file to see what the
//    values of payload and destination should be.
// 2. run `forge test -vv` and write down the addresses and payloads of each voucher.
// 3. we need to use the script `gen-proofs.sh` here[1]. It originally has 2 vouchers/notices. Make it into 6.
//    Replace `PAYLOAD` and `MSG_SENDER` accordingly.
//    For Apple silicon users, use long duration of `sleep` command before `# Finish epoch`. For example, `sleep 10`.
// 4. `gen-proofs.sh` outputs a JSON file with proofs in base64 encoding. This tool[2] converts base64 to hex.
//    To install: `pip install base64-to-hex-converter`
//    To run: `python -m b64to16 epoch-state.json | jq > test/foundry/dapp/helpers/voucherProofs.json`
// 5. go to this directory by running `cd test/foundry/dapp/helpers`
// 6. run `npx ts-node genProof.ts` to generate Solidity version of proofs
// 7. set the boolean state variable `logVouchers` back to `false` in `CartesiDApp.t.sol`
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
    const array1 = p.keccakInHashesSiblings;
    const array2 = p.outputHashesInEpochSiblings;
    const lines : string[] = [
        '// SPDX-License-Identifier: UNLICENSED',
        '',
        'pragma solidity ^0.8.13;',
        '',
        '// THIS FILE WAS AUTOMATICALLY GENERATED BY `genProof.ts`.',
        '',
        'import {OutputValidityProof} from "contracts/library/LibOutputValidation.sol";',
        '',
        'library LibProof {',
        '    function getProof() internal pure returns (OutputValidityProof memory) {',
        `        uint256[${array1.length}] memory array1 = [${array1}];`,
        `        uint256[${array2.length}] memory array2 = [${array2}];`,
        `        bytes32[] memory keccakInHashesSiblings = new bytes32[](${array1.length});`,
        `        bytes32[] memory outputHashesInEpochSiblings = new bytes32[](${array2.length});`,
        `        for (uint256 i; i < ${array1.length}; ++i) { keccakInHashesSiblings[i] = bytes32(array1[i]); }`,
        `        for (uint256 i; i < ${array2.length}; ++i) { outputHashesInEpochSiblings[i] = bytes32(array2[i]); }`,
        `        return OutputValidityProof({`,
        `            epochInputIndex: ${p.epochInputIndex},`,
        `            outputIndex: ${p.outputIndex},`,
        `            outputHashesRootHash: ${p.outputHashesRootHash},`,
        `            vouchersEpochRootHash: ${p.vouchersEpochRootHash},`,
        `            noticesEpochRootHash: ${p.noticesEpochRootHash},`,
        `            machineStateHash: ${p.machineStateHash},`,
        '            keccakInHashesSiblings: keccakInHashesSiblings,',
        '            outputHashesInEpochSiblings: outputHashesInEpochSiblings',
        '        });',
        '    }',
        '}',
    ];
    return lines.join('\n');
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
