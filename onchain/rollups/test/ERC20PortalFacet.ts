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
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import {
    DebugFacet,
    DebugFacet__factory,
    ERC20PortalFacet,
    ERC20PortalFacet__factory,
} from "../src/types";
import { deployDiamond, getInputHash } from "./utils";

use(solidity);

describe("ERC20Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC20PortalFacet;
    var debugFacet: DebugFacet;
    let mockERC20: MockContract; //mock erc20

    var numberOfInputs = 0x1; // the machine starts with one input

    beforeEach(async () => {
        const diamond = await deployDiamond({ debug: true });
        [signer, signer2] = await ethers.getSigners();
        portalFacet = ERC20PortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);

        // Deploy a mock ERC-20 contract
        const CTSI = await deployments.getArtifact("IERC20");
        mockERC20 = await deployMockContract(signer, CTSI.abi);
    });

    it("erc20Deposit should revert if transferFrom returns false", async () => {
        await mockERC20.mock.transferFrom.returns(false);

        await expect(
            portalFacet.erc20Deposit(mockERC20.address, 50, "0x00"),
            "ERC20 deposit should revert if ERC20 transferFrom fails"
        ).to.be.revertedWith("ERC20 transferFrom failed");
    });

    it("erc20Deposit should emit events", async () => {
        await mockERC20.mock.transferFrom.returns(true);

        const erc20 = mockERC20.address;
        const sender = await signer.getAddress();
        const value = 15;
        const data = "0x00";

        expect(
            await portalFacet.erc20Deposit(erc20, value, data),
            "expect erc20Deposit function to emit ERC20Deposited event"
        )
            .to.emit(portalFacet, "ERC20Deposited")
            .withArgs(erc20, sender, value, data);
    });

    it("erc20Deposit should return the return value of LibInput.addInput()", async () => {
        await mockERC20.mock.transferFrom.returns(true);

        const header = keccak256(toUtf8Bytes("ERC20_Transfer"));
        const erc20 = mockERC20.address;
        const sender = await signer.getAddress();
        const value = 15;
        const data = "0x00";

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            ["bytes32", "address", "address", "uint", "bytes"],
            [header, sender, erc20, value, data]
        );

        // Calculate the input hash
        const block = await ethers.provider.getBlock("latest");
        const inputHash = getInputHash(
            input,
            portalFacet.address,
            block.number,
            block.timestamp,
            0x0,
            numberOfInputs
        );

        expect(
            await portalFacet.callStatic.erc20Deposit(erc20, value, data),
            "callStatic to check return value"
        ).to.equal(inputHash);
    });

    // erc20Withdrawals are tested in output facet
});
