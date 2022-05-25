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
import { Signer } from "ethers";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import {
    DebugFacet,
    DebugFacet__factory,
    ERC721PortalFacet,
    ERC721PortalFacet__factory,
    InputFacet,
    InputFacet__factory,
    SimpleNFT,
} from "../src/types";
import { deployDiamond, deploySimpleNFT, getInputHash } from "./utils";

use(solidity);

describe("ERC721Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC721PortalFacet;
    var debugFacet: DebugFacet;
    let inputFacet: InputFacet;
    let simpleNFT: SimpleNFT;
    let tokenIds: number[] = [42, 123];
    const invalidTokenId = 99;
    var numberOfInputs = 0x1; // the machine starts with one input

    const safeTransferFrom = "safeTransferFrom(address,address,uint256)";
    const safeTransferFromWithData =
        "safeTransferFrom(address,address,uint256,bytes)";
    const inputHeader = keccak256(toUtf8Bytes("ERC721_Transfer"));

    beforeEach(async () => {
        const diamond = await deployDiamond({ debug: true });
        [signer, signer2] = await ethers.getSigners();
        portalFacet = ERC721PortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);
        inputFacet = InputFacet__factory.connect(diamond.address, signer);

        // Deploy a simple ERC-721 contract
        simpleNFT = await deploySimpleNFT({ tokenIds });
    });

    it("Check if the simple NFT contract was created correctly", async () => {
        const minter = await signer.getAddress();

        expect(
            await simpleNFT.balanceOf(minter),
            "Check number of tokens possessed by the minter"
        ).to.equal(tokenIds.length);

        for (const tokenId of tokenIds) {
            expect(
                await simpleNFT.ownerOf(tokenId),
                "Check owner of all tokens to be the minter"
            ).to.equal(minter);
        }

        await expect(
            simpleNFT.ownerOf(invalidTokenId),
            "Check owner of nonexistent token"
        ).to.be.revertedWith("ERC721: owner query for nonexistent token");
    });

    it("If addInput reverts, no token should be transfered", async () => {
        const tokenId = tokenIds[1];
        const data = Buffer.from("a".repeat(258), "utf-8");

        await expect(
            simpleNFT[safeTransferFromWithData](
                await signer.getAddress(),
                portalFacet.address,
                tokenId,
                data
            ),
            "Invalid input should revert transaction"
        ).to.be.revertedWith("input len: [0,driveSize]");
    });

    it("Transfer an NFT through the ERC-721 Portal and check for events", async () => {
        const tokenId = tokenIds[0];
        const data = "0xdeadbeef";

        // First, we make signer transfer the token to signer2
        await simpleNFT[safeTransferFrom](
            await signer.getAddress(),
            await signer2.getAddress(),
            tokenId
        );

        expect(
            await simpleNFT.ownerOf(tokenId),
            "Check that the new owner of token is signer2"
        ).to.equal(await signer2.getAddress());

        // Then, we allow signer to transfer the token from signer2
        // This is to make the `operator` field from the `ERC721Received` event different
        // from the `from` field (and make sure that it is handled correctly)
        await simpleNFT
            .connect(signer2)
            .approve(await signer.getAddress(), tokenId);

        expect(
            await simpleNFT.getApproved(tokenId),
            "Check that signer is allowed to transfer token on behalf of signer2"
        ).to.equal(await signer.getAddress());

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            ["bytes32", "address", "address", "address", "uint", "bytes"],
            [
                inputHeader,
                simpleNFT.address, // ERC721
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenId,
                data,
            ]
        );

        const tx = await simpleNFT[safeTransferFromWithData](
            await signer2.getAddress(), // from
            portalFacet.address, // to
            tokenId,
            data
        );

        const receipt = await tx.wait();

        expect(receipt.status, "Check if transaction succeeded").to.equal(1);

        // Get the block where the transaction was included so that
        // we can extract the `timestamp` field from it
        const block = await ethers.provider.getBlock(receipt.blockNumber);

        // Finally, we transfer the token from signer2 to the portal
        // We pass an arbitrary `data` field to check for the correct emission
        // of a `ERC721Received` event by the portal
        expect(tx, "Check for events")
            .to.emit(portalFacet, "ERC721Received")
            .withArgs(
                simpleNFT.address, // ERC721
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenId,
                data
            )
            .to.emit(inputFacet, "InputAdded")
            .withArgs(
                0, // epochNumber
                numberOfInputs, // inputIndex
                portalFacet.address, // sender
                block.timestamp, // timestamp
                input
            );
    });

    it("erc721Withdrawal should revert if not called by the Rollups contract", async () => {
        const erc721 = simpleNFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];

        // Transfer the token to the portal facet
        await simpleNFT[safeTransferFrom](
            receiver,
            portalFacet.address,
            tokenId
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint"],
            [erc721, receiver, tokenId]
        );

        await expect(
            portalFacet.connect(signer2).erc721Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("erc721Withdrawal should revert if safeTransferFrom fails", async () => {
        const erc721 = simpleNFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint"],
            [erc721, receiver, tokenId]
        );

        // The portal does not have the token, so safeTransferFrom will fail
        await expect(
            debugFacet._erc721Withdrawal(data),
            "Expect withdrawal to revert"
        ).to.be.revertedWith(
            "ERC721: transfer caller is not owner nor approved"
        );
    });

    it("erc721Withdrawal should emit ERC721Withdrawn and return true", async () => {
        const erc721 = simpleNFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];

        // Transfer the token to the portal facet
        await simpleNFT[safeTransferFrom](
            receiver,
            portalFacet.address,
            tokenId
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint"],
            [erc721, receiver, tokenId]
        );

        // callStatic to check the return value
        expect(
            await debugFacet.callStatic._erc721Withdrawal(data),
            "Check the return value of `erc721Withdrawal`"
        ).to.equal(true);

        // check emitted event
        expect(
            await debugFacet._erc721Withdrawal(data),
            "Check if `ERC721Withdrawn` is emmitted"
        )
            .to.emit(portalFacet, "ERC721Withdrawn")
            .withArgs(erc721, receiver, tokenId);
    });
});
