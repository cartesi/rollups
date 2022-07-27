// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { deployments, ethers } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { BytesLike } from "@ethersproject/bytes";
import { keccak256 } from "ethers/lib/utils";
import {
    DebugFacet,
    DebugFacet__factory,
    FeeManagerFacet,
    FeeManagerFacet__factory,
    OutputFacet,
    OutputFacet__factory,
    SimpleToken,
    SimpleToken__factory,
} from "../src/types";
import { deployDiamond, getState } from "./utils";
import epochStateN from "./outputProofs/decoded_proof_notice.json";
import epochStateV from "./outputProofs/decoded_proof_voucher.json";

use(solidity);

// How to update voucher proofs:
// 1. uncomment all 4 `console.log` statements and run to see what the values of payload and destination should be.
// 2. we need to use the script `gen-proofs.sh` here[1]. It originally has 2 vouchers/notices. Make it into 4.
//    Replace all 4 `PAYLOAD` and `MSG_SENDER`.
//    For Apple silicon users, use long duration of `sleep` command before `# Finish epoch`. For example, `sleep 10`.
// 3. `gen-proofs.sh` outputs a JSON file with proofs in base64 encoding. This tool[2] converts base64 to hex.
//    To install: `pip install base64-to-hex-converter`
//    To run: `python -m b64to16 proof.json`
// 4. import the decoded hex proof files for vouchers.
//
// ref links:
// [1]: https://github.com/cartesi-corp/machine-emulator/tree/feature/gen-proofs/tools/gen-proofs
// [2]: https://pypi.org/project/base64-to-hex-converter/
//
// Note: the script uses the same payload for both a voucher and a notice. But keep in mind that the encodings of
//       notices and vouchers are different. We use 2 notices from the original script. The notice proofs will not change.

describe("Output Facet", () => {
    let enableStateFold = process.env["STATE_FOLD_TEST"];

    let signers: Signer[];
    let outputFacet: OutputFacet;
    let feeManagerFacet: FeeManagerFacet;
    var debugFacet: DebugFacet;

    let simpleContractAddress: string;
    let _destination: string;
    let _payload: string;
    let encodedVoucher: string;
    let encodedNotice: string;

    const initialSupply = 1000000;
    let simpleToken: SimpleToken;

    const setupTest = deployments.createFixture(
        async ({ deployments, ethers }, options) => {
            const diamond = await deployDiamond({ debug: true });
            signers = await ethers.getSigners();

            outputFacet = OutputFacet__factory.connect(
                diamond.address,
                signers[0]
            );
            debugFacet = DebugFacet__factory.connect(
                diamond.address,
                signers[0]
            );
            feeManagerFacet = FeeManagerFacet__factory.connect(
                diamond.address,
                signers[0]
            );

            // deploy a simple contract to execute
            const simpleContract = await deployments.deploy("SimpleContract", {
                from: await signers[0].getAddress(),
                deterministicDeployment: true, // deployed address is calculated based on contract bytecode, constructor arguments, deployer address...
            });
            simpleContractAddress = simpleContract.address;

            // deploy simple token to test ERC20 withdrawals
            const SimpleToken_deploy = await deployments.deploy("SimpleToken", {
                from: await signers[0].getAddress(),
                args: [initialSupply],
                deterministicDeployment: true, // deployed address is calculated based on contract bytecode, constructor arguments, deployer address...
            });
            simpleToken = SimpleToken__factory.connect(
                SimpleToken_deploy.address,
                signers[0]
            );
        }
    );

    beforeEach(async () => {
        await deployments.fixture();
        await setupTest();
    });

    interface OutputValidityProof {
        epochIndex: number;
        inputIndex: number;
        outputIndex: number;
        outputHashesRootHash: BytesLike;
        vouchersEpochRootHash: BytesLike;
        noticesEpochRootHash: BytesLike;
        machineStateHash: BytesLike;
        keccakInHashesSiblings: BytesLike[];
        outputHashesInEpochSiblings: BytesLike[];
    }

    const iface = new ethers.utils.Interface([
        "function simple_function() public pure returns (string memory)",
        "function simple_function(bytes32) public pure returns (string memory)",
        "function nonExistent() public",
        "function transfer(address,uint256) public returns (bool)",
    ]);

    // create output validity proof for notice0
    let noticeProof = setupNoticeProof(0);
    let notice0 = "0xdeadbeef";
    encodedNotice = ethers.utils.defaultAbiCoder.encode(["bytes"], [notice0]);
    // epochHashForNotice will be the same for notice0 and notice1
    let epochHashForNotice = keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                noticeProof.vouchersEpochRootHash,
                noticeProof.noticesEpochRootHash,
                noticeProof.machineStateHash,
            ]
        )
    );

    // create output validity proof for voucher0
    // voucher0 calls "simple_function()"
    let voucherProof = setupVoucherProof(0);
    // encodedVoucher will be assigned later in Initialization

    // create output validity proof for voucher1
    // voucher1 calls "simple_function(bytes32)"
    let voucherProof1 = setupVoucherProof(1);
    // create output validity proof for voucher2
    // voucher2 calls "nonExistent()"
    let voucherProof2 = setupVoucherProof(2);
    // create output validity proof for voucher3
    // voucher3 calls "SimpleToken.transfer(address,uint256)"
    let voucherProof3 = setupVoucherProof(3);
    // all 4 vouchers are in epoch0, so epochHashForVoucher will be the same for all 4 vouchers
    let epochHashForVoucher = keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                voucherProof.vouchersEpochRootHash,
                voucherProof.noticesEpochRootHash,
                voucherProof.machineStateHash,
            ]
        )
    );

    it("check signer address", async () => {
        expect(
            await signers[0].getAddress(),
            "Failed to use Hardhat default signer, please unset user defined MNEMONIC env variable"
        ).to.equal("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    });

    it("Initialization", async () => {
        _destination = simpleContractAddress;
        _payload = iface.encodeFunctionData("simple_function()");
        encodedVoucher = ethers.utils.defaultAbiCoder.encode(
            ["uint", "bytes"],
            [_destination, _payload]
        );
        // console.log(_destination, _payload);

        expect(
            await outputFacet.getNumberOfFinalizedEpochs(),
            "check initial epoch number"
        ).to.equal(0);
    });

    // *************** Testing Notices *************** //

    /// ***test function isValidNoticeProof()*** ///
    it("testing function isValidNoticeProof()", async () => {
        await outputFacet.isValidNoticeProof(
            encodedNotice,
            epochHashForNotice,
            noticeProof
        );
    });

    it("isValidNoticeProof() should revert when _epochHash doesn't match", async () => {
        await expect(
            outputFacet.isValidNoticeProof(
                encodedNotice,
                ethers.utils.formatBytes32String("invalidEpochHash"),
                noticeProof
            )
        ).to.be.revertedWith("epochHash incorrect");
    });

    it("isValidNoticeProof() should revert when outputsEpochRootHash doesn't match", async () => {
        let tempInputIndex = noticeProof.inputIndex;
        noticeProof.inputIndex = 10;
        await expect(
            outputFacet.isValidNoticeProof(
                encodedNotice,
                epochHashForNotice,
                noticeProof
            )
        ).to.be.revertedWith("outputsEpochRootHash incorrect");
        // restore noticeProof
        noticeProof.inputIndex = tempInputIndex;
    });

    it("isValidNoticeProof() should revert when outputHashesRootHash doesn't match", async () => {
        let tempNoticeIndex = noticeProof.outputIndex;
        noticeProof.outputIndex = 10;
        await expect(
            outputFacet.isValidNoticeProof(
                encodedNotice,
                epochHashForNotice,
                noticeProof
            )
        ).to.be.revertedWith("outputHashesRootHash incorrect");
        // restore noticeProof
        noticeProof.outputIndex = tempNoticeIndex;
    });

    /// ***test function validateNotice()*** ///
    it("validateNotice(): valid notices should return true", async () => {
        // DebugFacet._onNewEpoch() should be called first to push some epochHashes
        // before calling OutputFacet.validateNotice()
        await debugFacet._onNewEpochOutput(epochHashForNotice);
        expect(
            await outputFacet.callStatic.validateNotice(notice0, noticeProof)
        ).to.equal(true);
    });

    it("validateNotice() should revert if proof is not valid", async () => {
        await debugFacet._onNewEpochOutput(epochHashForNotice);
        let invalidNotice = "0xbeaf";
        await expect(outputFacet.validateNotice(invalidNotice, noticeProof)).to
            .be.reverted;
    });

    it("testing notice1", async () => {
        // create output validity proof for notice1
        let noticeProof1 = setupNoticeProof(1);

        let notice1 = "0xbeefdead";
        let encodedNotice1 = ethers.utils.defaultAbiCoder.encode(
            ["bytes"],
            [notice1]
        );

        // test isValidNoticeProof()
        await outputFacet.isValidNoticeProof(
            encodedNotice1,
            epochHashForNotice,
            noticeProof1
        );

        // test validateNotice()
        await debugFacet._onNewEpochOutput(epochHashForNotice);
        expect(
            await outputFacet.callStatic.validateNotice(notice1, noticeProof1)
        ).to.equal(true);
    });

    // *************** Testing Vouchers *************** //

    /// ***test function isValidVoucherProof()*** ///
    it("testing function isValidVoucherProof()", async () => {
        await outputFacet.isValidVoucherProof(
            encodedVoucher,
            epochHashForVoucher,
            voucherProof
        );
    });

    it("isValidVoucherProof() should revert when _epochHash doesn't match", async () => {
        await expect(
            outputFacet.isValidVoucherProof(
                encodedVoucher,
                ethers.utils.formatBytes32String("invalidEpochHash"),
                voucherProof
            )
        ).to.be.revertedWith("epochHash incorrect");
    });

    it("isValidVoucherProof() should revert when outputsEpochRootHash doesn't match", async () => {
        let tempInputIndex = voucherProof.inputIndex;
        voucherProof.inputIndex = 10;
        await expect(
            outputFacet.isValidVoucherProof(
                encodedVoucher,
                epochHashForVoucher,
                voucherProof
            )
        ).to.be.revertedWith("outputsEpochRootHash incorrect");
        // restore voucherProof
        voucherProof.inputIndex = tempInputIndex;
    });

    it("isValidVoucherProof() should revert when outputHashesRootHash doesn't match", async () => {
        let tempVoucherIndex = voucherProof.outputIndex;
        voucherProof.outputIndex = 10;
        await expect(
            outputFacet.isValidVoucherProof(
                encodedVoucher,
                epochHashForVoucher,
                voucherProof
            )
        ).to.be.revertedWith("outputHashesRootHash incorrect");
        // restore voucherProof
        voucherProof.outputIndex = tempVoucherIndex;
    });

    /// ***test function executeVoucher()*** ///

    it("executeVoucher(): execute SimpleContract.simple_function()", async () => {
        await debugFacet._onNewEpochOutput(epochHashForVoucher);
        expect(
            await outputFacet.callStatic.executeVoucher(
                _destination,
                _payload,
                voucherProof
            )
        ).to.equal(true);
    });

    it("executeVoucher(): execute SimpleContract.simple_function(bytes32)", async () => {
        let _payload_new = iface.encodeFunctionData(
            "simple_function(bytes32)",
            [ethers.utils.formatBytes32String("hello")]
        );
        // console.log(_destination, _payload_new);

        await debugFacet._onNewEpochOutput(epochHashForVoucher);
        expect(
            await outputFacet.callStatic.executeVoucher(
                _destination,
                _payload_new,
                voucherProof1
            )
        ).to.equal(true);
    });

    it("executeVoucher(): should return false if the function to be executed failed (in this case the function does NOT exist)", async () => {
        let _payload_new = iface.encodeFunctionData("nonExistent()");
        // console.log(_destination, _payload_new);

        await debugFacet._onNewEpochOutput(epochHashForVoucher);
        expect(
            await outputFacet.callStatic.executeVoucher(
                _destination,
                _payload_new,
                voucherProof2
            )
        ).to.equal(false);
    });

    it("executeVoucher() should revert if voucher has already been executed", async () => {
        await debugFacet._onNewEpochOutput(epochHashForVoucher);
        await outputFacet.executeVoucher(_destination, _payload, voucherProof);
        await expect(
            outputFacet.executeVoucher(_destination, _payload, voucherProof)
        ).to.be.revertedWith("re-execution not allowed");
    });

    it("executeVoucher() should revert if proof is not valid", async () => {
        await debugFacet._onNewEpochOutput(epochHashForVoucher);
        let _payload_new = iface.encodeFunctionData("nonExistent()");
        await expect(
            outputFacet.executeVoucher(_destination, _payload_new, voucherProof)
        ).to.be.reverted;
    });

    /// ***test function getBitMaskPosition()*** ///
    it("testing function getBitMaskPosition()", async () => {
        const _voucher = 123;
        const _input = 456;
        const _epoch = 789;
        expect(
            await outputFacet.getBitMaskPosition(_voucher, _input, _epoch)
        ).to.equal(
            BigInt(_voucher) * BigInt(2 ** 128) +
                BigInt(_input) * BigInt(2 ** 64) +
                BigInt(_epoch)
        );
    });

    /// ***test function getIntraDrivePosition()*** ///
    it("testing function getIntraDrivePosition()", async () => {
        const _index = 10;
        const _log2Size = 4;
        expect(
            await outputFacet.getIntraDrivePosition(_index, _log2Size)
        ).to.equal(_index * 2 ** _log2Size);
    });

    /// ***test function getNumberOfFinalizedEpochs() and onNewEpoch()*** ///
    it("simulate calls to onNewEpoch() to test if getNumberOfFinalizedEpochs() increases", async () => {
        await debugFacet._onNewEpochOutput(
            ethers.utils.formatBytes32String("hello")
        );
        expect(await outputFacet.getNumberOfFinalizedEpochs()).to.equal(1);

        await debugFacet._onNewEpochOutput(
            ethers.utils.formatBytes32String("hello2")
        );
        expect(await outputFacet.getNumberOfFinalizedEpochs()).to.equal(2);

        await debugFacet._onNewEpochOutput(
            ethers.utils.formatBytes32String("hello3")
        );
        expect(await outputFacet.getNumberOfFinalizedEpochs()).to.equal(3);
    });

    /// ***test function getVoucherMetadataLog2Size()*** ///
    it("testing function getVoucherMetadataLog2Size()", async () => {
        expect(await outputFacet.getVoucherMetadataLog2Size()).to.equal(21);
    });

    /// ***test function getEpochVoucherLog2Size()*** ///
    it("testing function getEpochVoucherLog2Size()", async () => {
        expect(await outputFacet.getEpochVoucherLog2Size()).to.equal(37);
    });

    /// ***test function executeVoucher() for bad destination*** ///
    it("test function executeVoucher() for bad destination", async () => {
        const bankAddress = await feeManagerFacet.getFeeManagerBank();
        await expect(
            outputFacet.executeVoucher(bankAddress, _payload, voucherProof),
            "executing voucher for bank"
        ).to.be.revertedWith("bad destination");
    });

    // test executing vouchers that withdraw ERC20 tokens
    it("test erc20 withdrawal voucher", async () => {
        // send erc20 from dapp to recipient
        let recipient = await signers[1].getAddress();

        let destination_erc20 = simpleToken.address;
        let amount_erc20 = 7;
        let payload_erc20 = iface.encodeFunctionData(
            "transfer(address,uint256)",
            [recipient, amount_erc20]
        );
        // console.log(destination_erc20, payload_erc20);

        // enter new epoch
        await debugFacet._onNewEpochOutput(epochHashForVoucher);

        // fail if dapp doesn't have enough balance
        expect(
            await outputFacet.callStatic.executeVoucher(
                destination_erc20,
                payload_erc20,
                voucherProof3
            )
        ).to.equal(false);

        // fund dapp
        let dapp_init_balance = 100;
        await simpleToken.transfer(outputFacet.address, dapp_init_balance);

        // now it succeeds
        expect(
            await outputFacet.callStatic.executeVoucher(
                destination_erc20,
                payload_erc20,
                voucherProof3
            )
        ).to.equal(true);
        // modify state
        await outputFacet.executeVoucher(
            destination_erc20,
            payload_erc20,
            voucherProof3
        );
        expect(
            await simpleToken.balanceOf(recipient),
            "check recipient's balance"
        ).to.equal(amount_erc20);
        expect(
            await simpleToken.balanceOf(outputFacet.address),
            "check dapp's balance"
        ).to.equal(dapp_init_balance - amount_erc20);
    });

    /// ***test foldable*** ///
    if (enableStateFold) {
        it("testing output foldable", async () => {
            /// ***test case 1 - initial check
            let initialState = JSON.stringify(outputFacet.address);
            let state = JSON.parse(await getState(initialState));

            // initial check, executed vouchers should be empty
            expect(
                JSON.stringify(state.vouchers) == "{}",
                "initial check"
            ).to.equal(true);

            /// ***test case 2 - voucher0 executed
            await debugFacet._onNewEpochOutput(epochHashForVoucher);
            await outputFacet.executeVoucher(
                _destination,
                _payload,
                voucherProof
            );

            state = JSON.parse(await getState(initialState));

            // vouchers look like { '0': { '0': { '0': true } } }
            // format: {voucher_index: {input_index: {epoch_index:}}}
            expect(
                Object.keys(state.vouchers).length,
                "should have 1 executed voucher"
            ).to.equal(1);
            expect(
                state.vouchers[0][0][0],
                "the first voucher is executed successfully"
            ).to.equal(true);

            /// ***test case 3 - execute voucher1
            let _payload_new = iface.encodeFunctionData(
                "simple_function(bytes32)",
                [ethers.utils.formatBytes32String("hello")]
            );

            await outputFacet.executeVoucher(
                _destination,
                _payload_new,
                voucherProof1
            );

            state = JSON.parse(await getState(initialState));

            // vouchers look like { '0': { '0': { '0': true }, '1': { '0': true } } }
            // format: {voucher_index: {input_index: {epoch_index:}}}
            expect(
                Object.keys(state.vouchers[0]).length,
                "should have 2 executed vouchers generated by 2 inputs"
            ).to.equal(2);
            expect(
                state.vouchers[0][1][0],
                "execute the second voucher"
            ).to.equal(true);

            /// ***test case 4 - execute voucher2
            _payload_new = iface.encodeFunctionData("nonExistent()");
            await debugFacet._onNewEpochOutput(epochHashForVoucher);
            await outputFacet.executeVoucher(
                _destination,
                _payload_new,
                voucherProof2
            );

            state = JSON.parse(await getState(initialState));

            // since the execution was failed (function doesn't exist), everything should remain the same
            expect(
                Object.keys(state.vouchers).length,
                "only 1 outputIndex"
            ).to.equal(1);
            expect(
                Object.keys(state.vouchers[0]).length,
                "2 inputIndexes"
            ).to.equal(2);
            expect(
                Object.keys(state.vouchers[0][0]).length,
                "1 epochIndex"
            ).to.equal(1);
            expect(
                Object.keys(state.vouchers[0][1]).length,
                "another 1 epochIndex"
            ).to.equal(1);

            /// ***test case 5 - re-execute an already executed voucher
            /// ***and test case 6 - proof not valid
            /// after these 2 failure cases, the executed vouchers should remain the same
            await expect(
                outputFacet.executeVoucher(
                    _destination,
                    _payload,
                    voucherProof
                ),
                "already executed, should revert"
            ).to.be.revertedWith("re-execution not allowed");

            _payload_new = iface.encodeFunctionData("nonExistent()");
            await expect(
                outputFacet.executeVoucher(
                    _destination,
                    _payload_new,
                    voucherProof
                ),
                "proof not valid, should revert"
            ).to.be.reverted;

            state = JSON.parse(await getState(initialState));
            expect(
                Object.keys(state.vouchers).length,
                "still only 1 outputIndex"
            ).to.equal(1);
            expect(
                Object.keys(state.vouchers[0]).length,
                "still 2 inputIndexes"
            ).to.equal(2);
            expect(
                Object.keys(state.vouchers[0][0]).length,
                "still 1 epochIndex"
            ).to.equal(1);
            expect(
                Object.keys(state.vouchers[0][1]).length,
                "still another 1 epochIndex"
            ).to.equal(1);
        });
    }

    // ***helper function*** //
    function setupNoticeProof(inputIndex: number): OutputValidityProof {
        let noticeDataKeccakInHashes =
            epochStateN.processedInputs[inputIndex].acceptedData.notices[0]
                .keccakInNoticeHashes;
        let noticeHashesInEpoch =
            epochStateN.processedInputs[inputIndex].noticeHashesInEpoch
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
            epochIndex: 0,
            inputIndex: inputIndex,
            outputIndex: 0,
            outputHashesRootHash: noticeDataKeccakInHashes.rootHash.data,
            vouchersEpochRootHash:
                epochStateN.mostRecentVouchersEpochRootHash.data,
            noticesEpochRootHash:
                epochStateN.mostRecentNoticesEpochRootHash.data,
            machineStateHash: epochStateN.mostRecentMachineHash.data,
            keccakInHashesSiblings: siblingKeccakInHashesN.reverse(), // from top-down to bottom-up
            outputHashesInEpochSiblings: noticeHashesInEpochSiblingsN.reverse(),
        };
        return noticeProof;
    }

    function setupVoucherProof(inputIndex: number): OutputValidityProof {
        let voucherDataKeccakInHashes =
            epochStateV.processedInputs[inputIndex].acceptedData.vouchers[0]
                .keccakInVoucherHashes;
        let voucherHashesInEpoch =
            epochStateV.processedInputs[inputIndex].voucherHashesInEpoch
                .siblingHashes;
        var siblingKeccakInHashesV: BytesLike[] = [];
        var voucherHashesInEpochSiblings: BytesLike[] = [];
        voucherDataKeccakInHashes.siblingHashes.forEach((element) => {
            siblingKeccakInHashesV.push(element.data);
        });
        voucherHashesInEpoch.forEach((element) => {
            voucherHashesInEpochSiblings.push(element.data);
        });
        let voucherProof: OutputValidityProof = {
            epochIndex: 0,
            inputIndex: inputIndex,
            outputIndex: 0,
            outputHashesRootHash: voucherDataKeccakInHashes.rootHash.data,
            vouchersEpochRootHash:
                epochStateV.mostRecentVouchersEpochRootHash.data,
            noticesEpochRootHash:
                epochStateV.mostRecentNoticesEpochRootHash.data,
            machineStateHash: epochStateV.mostRecentMachineHash.data,
            keccakInHashesSiblings: siblingKeccakInHashesV.reverse(),
            outputHashesInEpochSiblings: voucherHashesInEpochSiblings.reverse(),
        };
        return voucherProof;
    }
});
