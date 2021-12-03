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
import { RollupsDebugFacet } from "../dist/src/types/RollupsDebugFacet";
import { RollupsDebugFacet__factory } from "../dist/src/types/factories/RollupsDebugFacet__factory";
import { getState } from "./getState";

use(solidity);

describe("Input Facet", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let signer: Signer;
    let inputFacet: InputFacet;
    let rollupsDebugFacet: RollupsDebugFacet;

    ///let enum starts from 0
    enum Phase {
        InputAccumulation = 0,
        AwaitingConsensus = 1,
        AwaitingDispute = 2,
    }

    beforeEach(async () => {
        await deployments.fixture();
        [signer] = await ethers.getSigners();

        const diamondAddress = (await deployments.get("CartesiRollupsDebug")).address;
        rollupsDebugFacet = RollupsDebugFacet__factory.connect(diamondAddress, signer);
        inputFacet = InputFacet__factory.connect(diamondAddress, signer);
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
        var input_150_bytes = Buffer.from("a".repeat(150), "utf-8");
        // one extra byte
        var input_129_bytes = Buffer.from("a".repeat(129), "utf-8");

        await expect(
            inputFacet.addInput(input_150_bytes),
            "input cant be bigger than drive"
        ).to.be.revertedWith("input len: (0,driveSize]");

        // input shouldnt fit because of one extra byte
        await expect(
            inputFacet.addInput(input_129_bytes),
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
        var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

        await inputFacet.addInput(input_64_bytes);
        await inputFacet.addInput(input_64_bytes);
        await inputFacet.addInput(input_64_bytes);

        expect(
            await inputFacet.getNumberOfInputs(),
            "Number of inputs should be zero, because non active inbox is empty"
        ).to.equal(0);

        // Enough time has passed...
        await rollupsDebugFacet._setInputAccumulationStart(0);

        await inputFacet.addInput(input_64_bytes);

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
        var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

        await expect(
            inputFacet.addInput(input_64_bytes),
            "should emit event InputAdded"
        )
            .to.emit(inputFacet, "InputAdded");
//            .withArgs(
//                0,
//                await signer.getAddress(),
//                (await ethers.provider.getBlock("latest")).timestamp + 1, // this is unstable
//                "0x" + input_64_bytes.toString("hex")
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
        var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

        // calculate input hash: keccak256(abi.encode(keccak256(metadata), keccak256(_input)))
        // metadata: abi.encode(msg.sender, block.timestamp)
        let metadata = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint", "uint", "uint"],
            [
                await signer.getAddress(),
                (await ethers.provider.getBlock("latest")).number,
                (await ethers.provider.getBlock("latest")).timestamp,
                0x0,
                0x0
            ]
        );
        let keccak_metadata = ethers.utils.keccak256(metadata);
        let keccak_input = ethers.utils.keccak256(input_64_bytes);
        let abi_metadata_input = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [keccak_metadata, keccak_input]
        );
        let input_hash = ethers.utils.keccak256(abi_metadata_input);

        expect(
            await inputFacet.callStatic.addInput(input_64_bytes),
            "use callStatic to view the return value"
        ).to.equal(input_hash);

        // test delegate
        if (enableDelegate) {
            await inputFacet.addInput(input_64_bytes);

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
            ).to.equal(input_64_bytes.toString());
        }
    });

    it("test getInput()", async () => {
        var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

        await inputFacet.addInput(input_64_bytes);

        // test for input box 0
        // calculate input hash again
        let metadata = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint", "uint", "uint"],
            [
                await signer.getAddress(),
                (await ethers.provider.getBlock("latest")).number,
                (await ethers.provider.getBlock("latest")).timestamp,
                0x0,
                0x0
            ]
        );

        let keccak_metadata = ethers.utils.keccak256(metadata);
        let keccak_input = ethers.utils.keccak256(input_64_bytes);
        let abi_metadata_input = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [keccak_metadata, keccak_input]
        );
        let input_hash = ethers.utils.keccak256(abi_metadata_input);

        // Enough time has passed...
        await rollupsDebugFacet._setInputAccumulationStart(0);

        // switch input boxes before testing getInput()
        await inputFacet.addInput(input_64_bytes);
        let block_epoch1 = await ethers.provider.getBlock("latest");

        expect(
            await inputFacet.getInput(0),
            "get the first value in input box 0"
        ).to.equal(input_hash);

        // test for input box 1
        // calculate input hash
        metadata = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint", "uint", "uint"],
            [
                await signer.getAddress(),
                (await ethers.provider.getBlock("latest")).number,
                (await ethers.provider.getBlock("latest")).timestamp,
                0x1,
                0x0
            ]
        );

        keccak_metadata = ethers.utils.keccak256(metadata);
        keccak_input = ethers.utils.keccak256(input_64_bytes);
        abi_metadata_input = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [keccak_metadata, keccak_input]
        );
        input_hash = ethers.utils.keccak256(abi_metadata_input);

        // We're accumulating inputs and enough time has passed...
        await rollupsDebugFacet._setCurrentPhase(0);
        await rollupsDebugFacet._setInputAccumulationStart(0);

        // switch input boxes before testing getInput()
        await inputFacet.addInput(input_64_bytes);

        expect(
            await inputFacet.getInput(0),
            "get the first value in input box 1"
        ).to.equal(input_hash);

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
            ).to.equal(input_64_bytes.toString());
        }
    });

    it("getCurrentInbox should return correct inbox", async () => {
        var input_64_bytes = Buffer.from("a".repeat(64), "utf-8");

        expect(
            await inputFacet.getCurrentInbox(),
            "current inbox should start as zero"
        ).to.equal(0);

        await inputFacet.addInput(input_64_bytes);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false"
        ).to.equal(0);

        // Enough time has passed...
        await rollupsDebugFacet._setInputAccumulationStart(0);

        await inputFacet.addInput(input_64_bytes);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox should change if notifyInput returns true"
        ).to.equal(1);

        await inputFacet.addInput(input_64_bytes);

        expect(
            await inputFacet.getCurrentInbox(),
            "inbox shouldnt change if notifyInput returns false (2)"
        ).to.equal(1);

        // there isn't a concept of input box in the delegate
    });
});
