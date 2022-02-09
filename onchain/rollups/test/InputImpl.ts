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
import { InputImpl__factory } from "../dist/src/types/factories/InputImpl__factory";
import { Signer } from "ethers";
import { InputImpl } from "../dist/src/types/InputImpl";
import { getState, getInputHash } from "./utils";

use(solidity);

describe("Input Implementation", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    /// for testing Rollups when modifiers are on, set this to true
    /// for testing Rollups when modifiers are off, set this to false
    let permissionModifiersOn = true;

    let signer: Signer;
    let inputImpl: InputImpl;
    let mockRollups: MockContract; //mock rollups implementation

    const log2Size = 7;

    beforeEach(async () => {
        await deployments.fixture(["RollupsImpl"]);
        [signer] = await ethers.getSigners();

        const Rollups = await deployments.getArtifact("Rollups");

        mockRollups = await deployMockContract(signer, Rollups.abi);

        const inputFactory = new InputImpl__factory(signer);

        inputImpl = await inputFactory.deploy(
            mockRollups.address,
            log2Size
        );
    });

    it("test constructor", async () => {
        const inputFactory = new InputImpl__factory(signer);

        let wrongLog2Size = 2;
        await expect(
            inputFactory.deploy(mockRollups.address, wrongLog2Size),
            "log2Size < 3"
        ).to.be.revertedWith("log size: [3,64]");

        wrongLog2Size = 65;
        await expect(
            inputFactory.deploy(mockRollups.address, wrongLog2Size),
            "log2Size > 64"
        ).to.be.revertedWith("log size: [3,64]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "shouldn't have any inputs right after constructor"
            ).to.equal(0);
        }
    });

    it("addInput should revert if input length == 0", async () => {
        await expect(
            inputImpl.addInput([]),
            "empty input should revert"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(
                state.inputs.length,
                "shouldn't have any inputs when adding empty inputs"
            ).to.equal(0);
        }
    });

    it("addInput should revert if input is larger than drive (log2Size)", async () => {
        var input_150_bytes = Buffer.from("a".repeat(150), "utf-8");
        // one extra byte
        var input_129_bytes = Buffer.from("a".repeat(129), "utf-8");

        await expect(
            inputImpl.addInput(input_150_bytes),
            "input cant be bigger than drive"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // input shouldnt fit because of one extra byte
        await expect(
            inputImpl.addInput(input_129_bytes),
            "input should still revert because metadata doesnt fit"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
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

        await mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(0);

        await inputImpl.addInput(input);
        await inputImpl.addInput(input);
        await inputImpl.addInput(input);

        expect(
            await inputImpl.getNumberOfInputs(),
            "Number of inputs should be zero, because non active inbox is empty"
        ).to.equal(0);

        await mockRollups.mock.notifyInput.returns(true);
        await mockRollups.mock.getCurrentEpoch.returns(1);

        await inputImpl.addInput(input);

        expect(
            await inputImpl.getNumberOfInputs(),
            "Number of inputs should be 3, because last addition changes the inbox"
        ).to.equal(3);

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
                epoch_number: "0x0",
            });
            let state = JSON.parse(await getState(initialState));
            expect(
                state.inputs.length,
                "now receiving inputs for epoch 1, getNumberOfInputs() reflects epoch 0"
            ).to.equal(3);

            initialState = JSON.stringify({
                input_address: inputImpl.address,
                epoch_number: "0x1",
            });
            state = JSON.parse(await getState(initialState));
            expect(state.inputs.length, "only 1 input for epoch 1").to.equal(1);
        }
    });

    it("emit event InputAdded", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        await mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(0);

        await expect(
            inputImpl.addInput(input),
            "should emit event InputAdded"
        )
            .to.emit(inputImpl, "InputAdded")
            .withArgs(
                0,
                await signer.getAddress(),
                (await ethers.provider.getBlock("latest")).timestamp + 1,
                "0x" + input.toString("hex")
            );

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
                epoch_number: "0x0",
            });

            let state = JSON.parse(await getState(initialState));

            expect(state.inputs.length, "only one input").to.equal(1);
        }
    });

    it("test return value of addInput()", async () => {
        var input = Buffer.from("a".repeat(64), "utf-8");

        await mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(0);

        let block = await ethers.provider.getBlock("latest");

        let inputHash = getInputHash(
            input,
            await signer.getAddress(),
            block.number,
            block.timestamp,
            0x0,
            0x0);

        expect(
            await inputImpl.callStatic.addInput(input),
            "use callStatic to view the return value"
        ).to.equal(inputHash);

        // test delegate
        if (enableDelegate) {
            await inputImpl.addInput(input);

            let initialState = JSON.stringify({
                input_address: inputImpl.address,
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
        await mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(0);

        await inputImpl.addInput(input);

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

        // switch input boxes before testing getInput()
        await mockRollups.mock.notifyInput.returns(true);
        await mockRollups.mock.getCurrentEpoch.returns(1);
        await inputImpl.addInput(input);
        let block_epoch1 = await ethers.provider.getBlock("latest");

        expect(
            await inputImpl.getInput(0),
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

        // switch input boxes before testing getInput()
        await mockRollups.mock.getCurrentEpoch.returns(2);
        await inputImpl.addInput(input);

        expect(
            await inputImpl.getInput(0),
            "get the first value in input box 1"
        ).to.equal(inputHash);

        // test delegate for epoch 1
        if (enableDelegate) {
            let initialState = JSON.stringify({
                input_address: inputImpl.address,
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

        await mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(0);

        expect(
            await inputImpl.getCurrentInbox(),
            "current inbox should start as zero"
        ).to.equal(0);

        await inputImpl.addInput(input);

        expect(
            await inputImpl.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false"
        ).to.equal(0);

        await mockRollups.mock.notifyInput.returns(true);
        await mockRollups.mock.getCurrentEpoch.returns(1);
        await inputImpl.addInput(input);

        expect(
            await inputImpl.getCurrentInbox(),
            "inbox should change if notifyInput returns true"
        ).to.equal(1);

        mockRollups.mock.notifyInput.returns(false);
        await mockRollups.mock.getCurrentEpoch.returns(1);
        await inputImpl.addInput(input);

        expect(
            await inputImpl.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false (2)"
        ).to.equal(1);

        // there isn't a concept of input box in the delegate
    });

    if (permissionModifiersOn) {
        it("onNewEpoch() can only be called by rollups", async () => {
            await expect(
                inputImpl.onNewEpoch(),
                "onNewEpoch() can only be called by rollups"
            ).to.be.revertedWith("Only rollups");
        });

        it("onNewInputAccumulation() can only be called by rollups", async () => {
            await expect(
                inputImpl.onNewInputAccumulation(),
                "onNewInputAccumulation() can only be called by rollups"
            ).to.be.revertedWith("Only rollups");
        });
    }

    if (!permissionModifiersOn) {
        it("test onNewInputAccumulation() with modifiers off", async () => {
            expect(
                await inputImpl.getCurrentInbox(),
                "initial box number"
            ).to.equal(0);

            await inputImpl.onNewInputAccumulation();
            expect(
                await inputImpl.getCurrentInbox(),
                "new input box, number should be 1"
            ).to.equal(1);

            await inputImpl.onNewInputAccumulation();
            expect(
                await inputImpl.getCurrentInbox(),
                "another new input box, number should be 0"
            ).to.equal(0);
        });

        it("test onNewEpoch() with modifiers off", async () => {
            // currentInputBox: 0
            expect(
                await inputImpl.getNumberOfInputs(),
                "initial box 1 should be empty"
            ).to.equal(0);

            var input = Buffer.from("a".repeat(64), "utf-8");
            await mockRollups.mock.notifyInput.returns(true);
            await mockRollups.mock.getCurrentEpoch.returns(0);
            // add input to box 1
            await inputImpl.addInput(input);

            // currentInputBox: 1
            expect(
                await inputImpl.getNumberOfInputs(),
                "initial box 0 should also be empty"
            ).to.equal(0);

            await mockRollups.mock.getCurrentEpoch.returns(1);
            // add 3 inputs to box 0
            await inputImpl.addInput(input);
            await mockRollups.mock.notifyInput.returns(false);
            await inputImpl.addInput(input);
            await inputImpl.addInput(input);

            // currentInputBox: 0
            expect(
                await inputImpl.getNumberOfInputs(),
                "box 1 should have 1 input"
            ).to.equal(1);

            // onNewEpoch() deletes box 1 and swap boxes
            await inputImpl.onNewEpoch();
            await inputImpl.onNewInputAccumulation();
            // currentInputBox: 1
            expect(
                await inputImpl.getNumberOfInputs(),
                "box 0 should have 3 input"
            ).to.equal(3);

            // onNewEpoch() deletes box 0 and swap boxes
            // now both boxes are empty
            await inputImpl.onNewEpoch();
            await inputImpl.onNewInputAccumulation();
            // currentInputBox: 0
            expect(
                await inputImpl.getNumberOfInputs(),
                "box 1 should have 0 input"
            ).to.equal(0);

            await inputImpl.onNewEpoch();
            await inputImpl.onNewInputAccumulation();
            // currentInputBox: 1
            expect(
                await inputImpl.getNumberOfInputs(),
                "box 0 should have 0 input"
            ).to.equal(0);
        });
    }
});
