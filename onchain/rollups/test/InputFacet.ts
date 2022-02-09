// Copyright (C) 2020 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { InputFacet } from "../dist/src/types/InputFacet";
import { InputFacet__factory } from "../dist/src/types/factories/InputFacet__factory";
import { DebugFacet } from "../dist/src/types/DebugFacet";
import { DebugFacet__factory } from "../dist/src/types/factories/DebugFacet__factory";
import { deployDiamond, getState, getInputHash } from "./utils";

use(solidity);

describe("Input Facet", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let signer: Signer;
    let inputFacet: InputFacet;
    let debugFacet: DebugFacet;

    ///let enum starts from 0
    enum Phase {
        InputAccumulation = 0,
        AwaitingConsensus = 1,
        AwaitingDispute = 2,
    }

    beforeEach(async () => {
        const diamond = await deployDiamond({ debug: true });
        [signer] = await ethers.getSigners();

        debugFacet = DebugFacet__factory.connect(diamond.address, signer);
        inputFacet = InputFacet__factory.connect(diamond.address, signer);
    });

    it("addInput should revert if input length == 0", async () => {
        await expect(
            inputFacet.addInput([]),
            "empty input should revert"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "shouldn't have any inputs when adding empty inputs"
            ).to.equal(0);
        }
    });

    it("addInput should revert if input is larger than drive (log2Size=7)", async () => {
        var input_300_bytes = Buffer.from("a".repeat(300), "utf-8");
        // one extra byte
        var input_257_bytes = Buffer.from("a".repeat(257), "utf-8");

        await expect(
            inputFacet.addInput(input_300_bytes),
            "input cant be bigger than drive"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // input shouldnt fit because of one extra byte
        await expect(
            inputFacet.addInput(input_257_bytes),
            "input should still revert because metadata doesnt fit"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "shouldn't have any inputs when adding inputs larger than the drive"
            ).to.equal(0);
        }
    });

    it("addInput should add input to inbox", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        await inputFacet.addInput(input);
        await inputFacet.addInput(input);
        await inputFacet.addInput(input);

        expect(
            await inputFacet.getNumberOfInputs(),
            "Number of inputs should be zero, because non active inbox is empty"
        ).to.equal(0);

        // Enough time has passed...
        await debugFacet._setInputAccumulationStart(0);

        await inputFacet.addInput(input);

        expect(
            await inputFacet.getNumberOfInputs(),
            "Number of inputs should be 3, because last addition changes the inbox"
        ).to.equal(3);

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x0",
            });
            let state = JSON.parse(await getState(initialState));
            expect(
                state.inputs.length,
                "now receiving inputs for epoch 1, getNumberOfInputs() reflects epoch 0"
            ).to.equal(3);

            initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x1",
            });
            state = JSON.parse(await getState(initialState));
            expect(state.inputs.length, "only 1 input for epoch 1").to.equal(1);
        }
    });

    it("emit event InputAdded", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        await expect(
            inputFacet.addInput(input),
            "should emit event InputAdded"
        )
            .to.emit(inputFacet, "InputAdded");
//            .withArgs(
//                0,
//                await signer.getAddress(),
//                (await ethers.provider.getBlock("latest")).timestamp + 1, // this is unstable
//                "0x" + input.toString("hex")
//            );

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(state.inputs.length, "only one input").to.equal(1);
        }
    });

    it("test return value of addInput()", async () => {
        const input = Buffer.from("a".repeat(64), "utf-8");
        let block = await ethers.provider.getBlock("latest");

        let inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block.number,
            block.timestamp,
            0x0,
            0x0);

        expect(
            await inputFacet.callStatic.addInput(input),
            "use callStatic to view the return value"
        ).to.equal(inputHash);

        // test delegate
        if (enableDelegate) {
            await inputFacet.addInput(input);

            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            // checking the input hash is essentially checking the
            // sender address, block timestamp, and payload
            // otherwise if hash is needed, follow the calculations above
            expect(
                state.inputs[0].sender,
                "check the recorded sender address"
            ).to.equal((await signer.getAddress()).toLowerCase());
            expect(
                parseInt(state.inputs[0].timestamp, 16), // from hex to dec
                "check the recorded timestamp"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);
            expect(
                Buffer.from(state.inputs[0].payload, "utf-8").toString(),
                "check the recorded payload"
            ).to.equal(input.toString());
        }
    });

    it("test getInput()", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        await inputFacet.addInput(input);

        // test for input box 0
        // calculate input hash again
        let block = await ethers.provider.getBlock("latest");

        let inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block.number,
            block.timestamp,
            0x0,
            0x0);

        // Enough time has passed...
        await debugFacet._setInputAccumulationStart(0);

        // switch input boxes before testing getInput()
        await inputFacet.addInput(input);
        let block_epoch1 = await ethers.provider.getBlock("latest");

        expect(
            await inputFacet.getInput(0),
            "get the first value in input box 0"
        ).to.equal(inputHash);

        // test for input box 1
        // calculate input hash
        block = await ethers.provider.getBlock("latest");

        inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block.number,
            block.timestamp,
            0x1,
            0x0);

        // We're accumulating inputs and enough time has passed...
        await debugFacet._setCurrentPhase(0);
        await debugFacet._setInputAccumulationStart(0);

        // switch input boxes before testing getInput()
        await inputFacet.addInput(input);

        expect(
            await inputFacet.getInput(0),
            "get the first value in input box 1"
        ).to.equal(inputHash);

        // test delegate for epoch 1
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputFacet.address,
                epoch_number: "0x1",
            });

            let state = JSON.parse(await getState(initialState));
            expect(
                state.inputs[0].sender,
                "check the recorded sender address for epoch 1"
            ).to.equal((await signer.getAddress()).toLowerCase());
            expect(
                parseInt(state.inputs[0].timestamp, 16), // from hex to dec
                "check the recorded timestamp for epoch 1"
            ).to.equal(block_epoch1.timestamp);
            expect(
                Buffer.from(state.inputs[0].payload, "utf-8").toString(),
                "check the recorded payload for epoch 1"
            ).to.equal(input.toString());
        }
    });

    it("getCurrentInbox should return correct inbox", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        expect(
            await inputFacet.getCurrentInbox(),
            "current inbox should start as zero"
        ).to.equal(0);

        await inputFacet.addInput(input);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false"
        ).to.equal(0);

        // Enough time has passed...
        await debugFacet._setInputAccumulationStart(0);

        await inputFacet.addInput(input);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox should change if notifyInput returns true"
        ).to.equal(1);

        await inputFacet.addInput(input);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false (2)"
        ).to.equal(1);

        // there isn't a concept of input box in the delegate
    });
});
