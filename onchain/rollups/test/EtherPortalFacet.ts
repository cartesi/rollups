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
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import {
    DebugFacet,
    DebugFacet__factory,
    EtherPortalFacet,
    EtherPortalFacet__factory,
} from "../src/types";
import { deployDiamond, getInputHash } from "./utils";
import { deployments, ethers } from "hardhat";

use(solidity);

describe("EtherPortal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: EtherPortalFacet;
    var debugFacet: DebugFacet;
    var numberOfInputs = 0x1; // the machine starts with one input

    beforeEach(async () => {
        await deployments.fixture();

        const diamond = await deployDiamond({ debug: true });
        [signer, signer2] = await ethers.getSigners();
        portalFacet = EtherPortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);
    });

    it("etherDeposit should emit events", async () => {
        const data = "0x00";
        const value = ethers.utils.parseEther("60");

        expect(
            await portalFacet.etherDeposit(data, { value: value }),
            "expect etherDeposit function to emit EtherDeposited event"
        )
            .to.emit(portalFacet, "EtherDeposited")
            .withArgs(await signer.getAddress(), value, data);
    });

    it("etherWithdrawal should revert if not called by the Rollups contract", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [await signer.getAddress(), 10]
        );
        await expect(
            portalFacet.connect(signer2).etherWithdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("etherWithdrawal should emit EtherWithdrawn and return true", async () => {
        // deposit ethers to portalFacet for enough balance to a successful transfer in etherWithdrawal()
        await portalFacet.etherDeposit("0x00", {
            value: ethers.utils.parseEther("10"),
        });

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [await signer.getAddress(), 10]
        );

        // callStatic check return value
        expect(await debugFacet.callStatic._etherWithdrawal(data)).to.equal(
            true
        );

        // check emitted event
        await expect(debugFacet._etherWithdrawal(data))
            .to.emit(portalFacet, "EtherWithdrawn")
            .withArgs(await signer.getAddress(), 10);
    });

    it("etherDeposit should return the return value of LibInput.addInput()", async () => {
        const header = keccak256(toUtf8Bytes("Ether_Transfer"));

        // create some random data
        let sender = await signer.getAddress();
        let value = ethers.utils.parseEther("10");
        let data = "0xdeadbeef";

        // ABI encode the input
        let input = ethers.utils.defaultAbiCoder.encode(
            ["bytes32", "address", "uint", "bytes"],
            [
                header, // keccak256("Ether_Transfer")
                sender, // msg.sender
                value, // msg.value
                data, // _data
            ]
        );

        // Calculate the input hash
        let block = await ethers.provider.getBlock("latest");
        let inputHash = getInputHash(
            input,
            portalFacet.address,
            block.number,
            block.timestamp,
            0x0,
            numberOfInputs
        );

        // check if input hashes are identical
        expect(
            await portalFacet.callStatic.etherDeposit(data, { value: value }),
            "callStatic to check return value"
        ).to.equal(inputHash);
    });
});
