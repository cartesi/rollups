// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import { solidity } from "ethereum-waffle";
import { BytesLike, Signer } from "ethers";
import {
    DebugFacet,
    DebugFacet__factory,
    InputFacet,
    InputFacet__factory,
    RollupsFacet__factory,
} from "../src/types";
import {
    deployDiamond,
    getState,
    getInputHash,
    increaseTimeAndMine,
} from "./utils";

use(solidity);

describe("Input Facet", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let signer: Signer;
    let inputFacet: InputFacet;
    let debugFacet: DebugFacet;
    let inputDuration: number;

    const NUM_OF_INITIAL_INPUTS = 1; // machine starts with one input
    let numberOfInputs: number;

    ///let enum starts from 0
    enum Phase {
        InputAccumulation = 0,
        AwaitingConsensus = 1,
        AwaitingDispute = 2,
    }

    async function addInputAndIncreaseCounter(input: BytesLike) {
        numberOfInputs++;
        return await inputFacet.addInput(input);
    }

    // Increase the current time in the network by just above
    // the input duration and force a block to be mined
    async function passInputAccumulationPeriod() {
        await increaseTimeAndMine(inputDuration + 1);
    }

    // Check the arguments of the last emitted `InputAdded` event
    // with `epochNumber` and `inputIndex` indexed arguments
    async function checkInputAddedEventArgs(
        epochNumber: number,
        inputIndex: number,
        sender: string,
        timestamp: number,
        input: Buffer
    ) {
        // we use ethers.js to query historic events, filtering by indexed event parameters
        // ref: https://docs.ethers.io/v5/single-page/#/v5/api/contract/contract/-%23-Contract--filters
        let eventFilter = inputFacet.filters.InputAdded(
            epochNumber,
            inputIndex
        );
        let event = await inputFacet.queryFilter(eventFilter);
        let eventArgs = event[0].args;

        expect(eventArgs.sender, "Input sender").to.equal(sender);
        expect(eventArgs.timestamp, "Input timestamp").to.equal(timestamp);
        expect(eventArgs.input, "Input").to.equal("0x" + input.toString("hex"));
    }

    beforeEach(async () => {
        await deployments.fixture();

        numberOfInputs = NUM_OF_INITIAL_INPUTS;

        const diamond = await deployDiamond({ debug: true });
        [signer] = await ethers.getSigners();

        debugFacet = DebugFacet__factory.connect(diamond.address, signer);
        inputFacet = InputFacet__factory.connect(diamond.address, signer);

        const rollupsFacet = RollupsFacet__factory.connect(
            diamond.address,
            signer
        );
        inputDuration = (await rollupsFacet.getInputDuration()).toNumber();
    });

    it("addInput should not revert if input length == 0", async () => {
        await addInputAndIncreaseCounter([]);

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "incorrect number of inputs after adding empty input"
            ).to.equal(numberOfInputs);
        }
    });

    it("addInput should revert if input is larger than drive (log2Size=7)", async () => {
        let input_300_bytes = Buffer.from("a".repeat(300), "utf-8");
        // one extra byte
        let input_257_bytes = Buffer.from("a".repeat(257), "utf-8");

        await expect(
            inputFacet.addInput(input_300_bytes),
            "input cant be bigger than drive"
        ).to.be.revertedWith("input len: [0,driveSize]");

        // input shouldnt fit because of one extra byte
        await expect(
            inputFacet.addInput(input_257_bytes),
            "input should still revert because metadata doesnt fit"
        ).to.be.revertedWith("input len: [0,driveSize]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "should not increase the number of inputs when adding inputs larger than the drive"
            ).to.equal(NUM_OF_INITIAL_INPUTS);
        }
    });

    it("addInput should add input to inbox", async () => {
        let input = Buffer.from("a".repeat(64), "utf-8");
        const numOfInputsToAdd = 3;

        for (let i = 0; i < numOfInputsToAdd; i++) {
            await addInputAndIncreaseCounter(input);
        }

        expect(
            await inputFacet.getNumberOfInputs(),
            "Number of inputs should be zero, because non active inbox is empty"
        ).to.equal(0);

        // Enough time has passed...
        await passInputAccumulationPeriod();

        await addInputAndIncreaseCounter(input);

        expect(
            await inputFacet.getNumberOfInputs(),
            "Now it's epoch 1, getNumberOfInputs() returns the number of inputs from epoch 0"
        ).to.equal(numOfInputsToAdd + NUM_OF_INITIAL_INPUTS);

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x0",
            });
            let state = JSON.parse(await getState(initialState));
            expect(
                state.inputs.length,
                "number of inputs doesn't match for epoch 0"
            ).to.equal(numOfInputsToAdd + NUM_OF_INITIAL_INPUTS);

            initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x1",
            });
            state = JSON.parse(await getState(initialState));
            expect(state.inputs.length, "only 1 input for epoch 1").to.equal(1);
        }
    });

    it("internal inputs should have DApps address as sender", async () => {
        let block = await ethers.provider.getBlock("latest");
        await passInputAccumulationPeriod();

        // switch input boxes before testing getInput()
        await addInputAndIncreaseCounter("0x00");

        let input = Buffer.from("".repeat(64), "utf-8");
        let sender = inputFacet.address;
        let epochNumber = 0x0;
        let inputIndex = NUM_OF_INITIAL_INPUTS - 1;

        let inputHash = getInputHash(
            input,
            sender,
            block.number,
            block.timestamp,
            epochNumber,
            inputIndex
        );

        expect(
            await inputFacet.getInput(inputIndex),
            "input hash doesnt match setup input"
        ).to.equal(inputHash);

        await checkInputAddedEventArgs(
            epochNumber,
            inputIndex,
            sender,
            block.timestamp,
            input
        );
    });

    it("emit event InputAdded", async () => {
        let input = Buffer.from("a".repeat(64), "utf-8");

        let tx = await addInputAndIncreaseCounter(input);
        let receipt = await tx.wait();
        let block = await ethers.provider.getBlock(receipt.blockNumber);

        let sender = await signer.getAddress();
        let epochNumber = 0x0;
        let inputIndex = numberOfInputs - 1; // latest input

        await checkInputAddedEventArgs(
            epochNumber,
            inputIndex,
            sender,
            block.timestamp,
            input
        );

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(state.inputs.length, "initial input + new input").to.equal(
                NUM_OF_INITIAL_INPUTS + 1
            );
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
            numberOfInputs
        );

        expect(
            await inputFacet.callStatic.addInput(input),
            "use callStatic to view the return value"
        ).to.equal(inputHash);

        // test delegate
        if (enableDelegate) {
            await addInputAndIncreaseCounter(input);

            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            // checking the input hash is essentially checking the
            // sender address, block timestamp, and payload
            // otherwise if hash is needed, follow the calculations above
            // during epoch 0, state.inputs[0] is an initial/default input

            // index of the lastly(newly) added input for epoch 0
            let index_input_epoch0 = NUM_OF_INITIAL_INPUTS;
            expect(
                state.inputs[index_input_epoch0].sender,
                "check the recorded sender address"
            ).to.equal((await signer.getAddress()).toLowerCase());
            expect(
                parseInt(state.inputs[index_input_epoch0].timestamp, 16), // from hex to dec
                "check the recorded timestamp"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);
            expect(
                Buffer.from(
                    state.inputs[index_input_epoch0].payload,
                    "utf-8"
                ).toString(),
                "check the recorded payload"
            ).to.equal(input.toString());
        }
    });

    it("test getInput()", async () => {
        let input = Buffer.from("a".repeat(64), "utf-8");
        await addInputAndIncreaseCounter(input);

        // test for input box 0
        // calculate input hash again
        let block = await ethers.provider.getBlock("latest");

        let inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block.number,
            block.timestamp,
            0x0,
            NUM_OF_INITIAL_INPUTS
        );

        // Enough time has passed...
        await passInputAccumulationPeriod();

        // switch input boxes before testing getInput()
        await addInputAndIncreaseCounter(input);
        let block_epoch1 = await ethers.provider.getBlock("latest");

        // index of the lastly(newly) added input for epoch 0
        let index_input_epoch0 = NUM_OF_INITIAL_INPUTS;
        expect(
            await inputFacet.getInput(index_input_epoch0),
            "input hash doesn't match for the newly added input in epoch 0"
        ).to.equal(inputHash);

        // test for input box 1
        // calculate input hash
        let index_input_epoch1 = 0;
        inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block_epoch1.number,
            block_epoch1.timestamp,
            0x1,
            index_input_epoch1
        );

        // We're accumulating inputs and enough time has passed...
        await debugFacet._setCurrentPhase(0);
        await passInputAccumulationPeriod();

        // add input just to switch input boxes before testing getInput()
        await inputFacet.addInput(input);

        expect(
            await inputFacet.getInput(index_input_epoch1),
            "get the first input in input box 1"
        ).to.equal(inputHash);

        // test delegate for epoch 1
        if (enableDelegate) {
            let initialState = JSON.stringify({
                dapp_contract_address: inputFacet.address,
                epoch_number: "0x1",
            });

            let state = JSON.parse(await getState(initialState));
            expect(
                state.inputs[index_input_epoch1].sender,
                "check the recorded sender address for epoch 1"
            ).to.equal((await signer.getAddress()).toLowerCase());
            expect(
                parseInt(state.inputs[index_input_epoch1].timestamp, 16), // from hex to dec
                "check the recorded timestamp for epoch 1"
            ).to.equal(block_epoch1.timestamp);
            expect(
                Buffer.from(
                    state.inputs[index_input_epoch1].payload,
                    "utf-8"
                ).toString(),
                "check the recorded payload for epoch 1"
            ).to.equal(input.toString());
        }
    });

    it("getCurrentInbox should return correct inbox", async () => {
        let input = Buffer.from("a".repeat(64), "utf-8");

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
        await passInputAccumulationPeriod();

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
