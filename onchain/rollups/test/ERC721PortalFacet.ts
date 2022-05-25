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
    ERC721PortalFacet,
    ERC721PortalFacet__factory,
} from "../src/types";
import { deployDiamond, getInputHash } from "./utils";

use(solidity);

describe("ERC721Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC721PortalFacet;
    var debugFacet: DebugFacet;
    let mockERC721: MockContract; //mock erc721
    var numberOfInputs = 0x1; // the machine starts with one input

    beforeEach(async () => {
        const diamond = await deployDiamond({ debug: true });
        [signer, signer2] = await ethers.getSigners();
        portalFacet = ERC721PortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);

        // Deploy a mock ERC-721 contract
        const IERC721 = await deployments.getArtifact("IERC721");
        mockERC721 = await deployMockContract(signer, IERC721.abi);
    });

//    it("erc721Deposit should revert if safeTransferFrom reverts", async () => {
//        const reason = "This cryptokitty is not available";
//        await mockERC721.mock[
//            "safeTransferFrom(address,address,uint256)"
//        ].revertsWithReason(reason);
//
//        const erc721 = mockERC721.address;
//        const tokenId = 50;
//        const data = "0x00";
//
//        await expect(
//            portalFacet.erc721Deposit(erc721, tokenId, data),
//            "ERC721 deposit should revert if ERC721 safeTransferFrom fails"
//        ).to.be.revertedWith(reason);
//    });
//
//    it("erc721Deposit should emit events", async () => {
//        await mockERC721.mock[
//            "safeTransferFrom(address,address,uint256)"
//        ].returns();
//
//        const erc721 = mockERC721.address;
//        const sender = await signer.getAddress();
//        const tokenId = 15;
//        const data = "0x00";
//
//        expect(
//            await portalFacet.erc721Deposit(erc721, tokenId, data),
//            "expect erc721Deposit function to emit ERC721Deposited event"
//        )
//            .to.emit(portalFacet, "ERC721Deposited")
//            .withArgs(erc721, sender, tokenId, data);
//    });
//
//    it("erc721Deposit should return LibInput.addInput(...)", async () => {
//        await mockERC721.mock[
//            "safeTransferFrom(address,address,uint256)"
//        ].returns();
//
//        const header = keccak256(toUtf8Bytes("ERC721_Transfer"));
//        const erc721 = mockERC721.address;
//        const sender = await signer.getAddress();
//        const tokenId = 15;
//        const data = "0x00";
//
//        // Encode input using the default ABI
//        const input = ethers.utils.defaultAbiCoder.encode(
//            ["bytes32", "address", "address", "uint", "bytes"],
//            [header, sender, erc721, tokenId, data]
//        );
//
//        // Calculate the input hash
//        const block = await ethers.provider.getBlock("latest");
//        const inputHash = getInputHash(
//            input,
//            portalFacet.address,
//            block.number,
//            block.timestamp,
//            0x0,
//            numberOfInputs
//        );
//
//        expect(
//            await portalFacet.callStatic.erc721Deposit(erc721, tokenId, data),
//            "callStatic to check return value"
//        ).to.equal(inputHash);
//    });

    it("erc721Withdrawal should revert if not called by the Rollups contract", async () => {
        const erc721 = mockERC721.address;
        const receiver = await signer.getAddress();
        const tokenId = 15;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [erc721, receiver, tokenId]
        );
        await expect(
            portalFacet.connect(signer2).erc721Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("erc721Withdrawal should emit ERC721Withdrawn and return true", async () => {
        await mockERC721.mock[
            "safeTransferFrom(address,address,uint256)"
        ].returns();

        const erc721 = mockERC721.address;
        const receiver = await signer.getAddress();
        const tokenId = 42;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [erc721, receiver, tokenId]
        );

        // callStatic check return value
        expect(await debugFacet.callStatic._erc721Withdrawal(data)).to.equal(
            true
        );

        // check emitted event
        await expect(debugFacet._erc721Withdrawal(data))
            .to.emit(portalFacet, "ERC721Withdrawn")
            .withArgs(erc721, receiver, tokenId);
    });
});
