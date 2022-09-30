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
import { BigNumber, Signer } from "ethers";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import {
    DebugFacet,
    DebugFacet__factory,
    ERC1155PortalFacet,
    ERC1155PortalFacet__factory,
    InputFacet,
    InputFacet__factory,
    SimpleSFT,
} from "../src/types";
import { deployDiamond, deploySimpleSFT, getInputHash } from "./utils";

use(solidity);

describe("ERC1155Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC1155PortalFacet;
    var debugFacet: DebugFacet;
    let inputFacet: InputFacet;
    let simpleSFT: SimpleSFT;
    let tokenIds: number[] = [42, 123];
    let tokenAmounts: number[] = [1, 100];
    const invalidTokenId = 99;
    var numberOfInputs = 0x1; // the machine starts with one input

    const safeTransferFromWithData =
        "safeTransferFrom(address,address,uint256,uint256,bytes)";
    const safeBatchTransferFromWithData =
        "safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)";
    const inputHeader = keccak256(toUtf8Bytes("ERC1155_Transfer"));

    beforeEach(async () => {
        await deployments.fixture();

        const diamond = await deployDiamond({ debug: true });
        [signer, signer2] = await ethers.getSigners();
        portalFacet = ERC1155PortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);
        inputFacet = InputFacet__factory.connect(diamond.address, signer);

        // Deploy a simple ERC-1155 contract
        simpleSFT = await deploySimpleSFT({
            tokenIds,
            tokenAmounts,
        });
    });

    it("Check if the simple ERC1155 contract was created correctly", async () => {
        const minter = await signer.getAddress();

        expect(
            await simpleSFT.balanceOfBatch(
                Array(tokenIds.length).fill(minter),
                tokenIds
            ),
            "Check number of tokens possessed by the minter"
        ).to.deep.equal(tokenAmounts.map((x) => BigNumber.from(x)));

        expect(
            await simpleSFT.balanceOf(minter, invalidTokenId),
            "Check balance of nonexistent token"
        ).to.equal(0);
    });

    it("If addInput reverts, no token should be transfered", async () => {
        const tokenId = tokenIds[1];
        const tokenAmount = tokenAmounts[1];
        const data = Buffer.from("a".repeat(516), "utf-8");

        await expect(
            simpleSFT[safeTransferFromWithData](
                await signer.getAddress(),
                portalFacet.address,
                tokenId,
                tokenAmount,
                data
            ),
            "Invalid input should revert transaction"
        ).to.be.revertedWith("input len: [0,driveSize]");
    });

    it("Transfer a token through the ERC-1155 Portal and check for events", async () => {
        const tokenId = tokenIds[0];
        const tokenAmount = tokenAmounts[0];
        const data = "0xdeadbeef";

        // First, we make signer transfer the token to signer2
        await simpleSFT[safeTransferFromWithData](
            await signer.getAddress(),
            await signer2.getAddress(),
            tokenId,
            tokenAmount,
            []
        );

        expect(
            await simpleSFT.balanceOf(await signer2.getAddress(), tokenId),
            "Check that the new owner of token is signer2"
        ).to.equal(tokenAmount);

        // Then, we allow signer to transfer the token from signer2
        // This is to make the `operator` field from the `ERC1155Received` event different
        // from the `from` field (and make sure that it is handled correctly)
        await simpleSFT
            .connect(signer2)
            .setApprovalForAll(await signer.getAddress(), true);

        expect(
            await simpleSFT.isApprovedForAll(
                await signer2.getAddress(),
                await signer.getAddress()
            ),
            "Check that signer is allowed to transfer tokens on behalf of signer2"
        ).to.be.true;

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            [
                "bytes32",
                "address",
                "address",
                "address",
                "uint",
                "uint",
                "bytes",
            ],
            [
                inputHeader,
                simpleSFT.address, // ERC1155
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenId,
                tokenAmount,
                data,
            ]
        );

        const tx = await simpleSFT[safeTransferFromWithData](
            await signer2.getAddress(), // from
            portalFacet.address, // to
            tokenId,
            tokenAmount,
            data
        );

        const receipt = await tx.wait();

        expect(receipt.status, "Check if transaction succeeded").to.equal(1);

        // Get the block where the transaction was included so that
        // we can extract the `timestamp` field from it
        const block = await ethers.provider.getBlock(receipt.blockNumber);

        // Finally, we transfer the token from signer2 to the portal
        // We pass an arbitrary `data` field to check for the correct emission
        // of a `ERC1155Received` event by the portal
        expect(tx, "Check for events")
            .to.emit(portalFacet, "ERC1155Received")
            .withArgs(
                simpleSFT.address, // ERC1155
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenId,
                tokenAmount,
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

    it("erc1155Withdrawal should revert if not called by the Rollups contract", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];
        const tokenAmount = tokenAmounts[0];
        const transferData = [];

        // Transfer the token to the portal facet
        await simpleSFT[safeTransferFromWithData](
            receiver,
            portalFacet.address,
            tokenId,
            tokenAmount,
            transferData
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint", "uint", "bytes"],
            [erc1155, receiver, tokenId, tokenAmount, transferData]
        );

        await expect(
            portalFacet.connect(signer2).erc1155Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("erc1155Withdrawal should revert if safeTransferFrom fails", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];
        const tokenAmount = tokenAmounts[0];
        const transferData = [];

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint", "uint", "bytes"],
            [erc1155, receiver, tokenId, tokenAmount, transferData]
        );

        // The portal does not have the token, so safeTransferFrom will fail
        await expect(
            debugFacet._erc1155Withdrawal(data),
            "Expect withdrawal to revert"
        ).to.be.revertedWith("ERC1155: insufficient balance for transfer");
    });

    it("erc1155Withdrawal should emit ERC1155Withdrawn and return true", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const tokenId = tokenIds[0];
        const tokenAmount = tokenAmounts[0];
        const transferData = [];

        // Transfer the token to the portal facet
        await simpleSFT[safeTransferFromWithData](
            receiver,
            portalFacet.address,
            tokenId,
            tokenAmount,
            transferData
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint", "uint", "bytes"],
            [erc1155, receiver, tokenId, tokenAmount, transferData]
        );

        // callStatic to check the return value
        expect(
            await debugFacet.callStatic._erc1155Withdrawal(data),
            "Check the return value of `erc1155Withdrawal`"
        ).to.equal(true);

        // check emitted event
        expect(
            await debugFacet._erc1155Withdrawal(data),
            "Check if `ERC1155Withdrawn` is emmitted"
        )
            .to.emit(portalFacet, "ERC1155Withdrawn")
            .withArgs(erc1155, receiver, tokenId, tokenAmount, transferData);
    });

    it("[Batch] Check if the token balances returned to original state", async () => {
        const minter = await signer.getAddress();

        expect(
            await simpleSFT.balanceOfBatch(
                Array(tokenIds.length).fill(minter),
                tokenIds
            ),
            "Check number of tokens possessed by the minter"
        ).to.deep.equal(tokenAmounts.map((x) => BigNumber.from(x)));
    });

    it("[Batch] If addInput reverts, no token should be transfered", async () => {
        const data = Buffer.from("a".repeat(516), "utf-8");

        await expect(
            simpleSFT[safeBatchTransferFromWithData](
                await signer.getAddress(),
                portalFacet.address,
                tokenIds,
                tokenAmounts,
                data
            ),
            "Invalid input should revert transaction"
        ).to.be.revertedWith("input len: [0,driveSize]");
    });

    it("[Batch] Transfer tokens through the ERC-1155 Portal and check for events", async () => {
        const data = "0xdeadbeef";

        // First, we make signer transfer the token to signer2
        await simpleSFT[safeBatchTransferFromWithData](
            await signer.getAddress(),
            await signer2.getAddress(),
            tokenIds,
            tokenAmounts,
            []
        );

        expect(
            await simpleSFT.balanceOfBatch(
                Array(tokenIds.length).fill(await signer2.getAddress()),
                tokenIds
            ),
            "Check that the new owner of tokens is signer2"
        ).to.deep.equal(tokenAmounts.map((x) => BigNumber.from(x)));

        // Then, we allow signer to transfer the token from signer2
        // This is to make the `operator` field from the `ERC1155Received` event different
        // from the `from` field (and make sure that it is handled correctly)
        await simpleSFT
            .connect(signer2)
            .setApprovalForAll(await signer.getAddress(), true);

        expect(
            await simpleSFT.isApprovedForAll(
                await signer2.getAddress(),
                await signer.getAddress()
            ),
            "Check that signer is allowed to transfer tokens on behalf of signer2"
        ).to.be.true;

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            [
                "bytes32",
                "address",
                "address",
                "address",
                "uint[]",
                "uint[]",
                "bytes",
            ],
            [
                inputHeader,
                simpleSFT.address, // ERC1155
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenIds,
                tokenAmounts,
                data,
            ]
        );

        const tx = await simpleSFT[safeBatchTransferFromWithData](
            await signer2.getAddress(), // from
            portalFacet.address, // to
            tokenIds,
            tokenAmounts,
            data
        );

        const receipt = await tx.wait();

        expect(receipt.status, "Check if transaction succeeded").to.equal(1);

        // Get the block where the transaction was included so that
        // we can extract the `timestamp` field from it
        const block = await ethers.provider.getBlock(receipt.blockNumber);

        // Finally, we transfer the token from signer2 to the portal
        // We pass an arbitrary `data` field to check for the correct emission
        // of a `ERC1155BatchReceived` event by the portal
        expect(tx, "Check for events")
            .to.emit(portalFacet, "ERC1155BatchReceived")
            .withArgs(
                simpleSFT.address, // ERC1155
                await signer.getAddress(), // operator
                await signer2.getAddress(), // from
                tokenIds,
                tokenAmounts,
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

    it("[Batch] erc1155BatchWithdrawal should revert if not called by the Rollups contract", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const transferData = [];

        // Transfer the token to the portal facet
        await simpleSFT[safeBatchTransferFromWithData](
            receiver,
            portalFacet.address,
            tokenIds,
            tokenAmounts,
            transferData
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint[]", "uint[]", "bytes"],
            [erc1155, receiver, tokenIds, tokenAmounts, transferData]
        );

        await expect(
            portalFacet.connect(signer2).erc1155BatchWithdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("[Batch] erc1155BatchWithdrawal should revert if safeBatchTransferFrom fails", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const transferData = [];

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint[]", "uint[]", "bytes"],
            [erc1155, receiver, tokenIds, tokenAmounts, transferData]
        );

        // The portal does not have the token, so safeTransferFrom will fail
        await expect(
            debugFacet._erc1155BatchWithdrawal(data),
            "Expect batch withdrawal to revert"
        ).to.be.revertedWith("ERC1155: insufficient balance for transfer");
    });

    it("[Batch] erc1155BatchWithdrawal should emit ERC1155BatchWithdrawn and return true", async () => {
        const erc1155 = simpleSFT.address;
        const receiver = await signer.getAddress();
        const transferData = [];

        // Transfer the token to the portal facet
        await simpleSFT[safeBatchTransferFromWithData](
            receiver,
            portalFacet.address,
            tokenIds,
            tokenAmounts,
            transferData
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["address", "address", "uint[]", "uint[]", "bytes"],
            [erc1155, receiver, tokenIds, tokenAmounts, transferData]
        );

        // callStatic to check the return value
        expect(
            await debugFacet.callStatic._erc1155BatchWithdrawal(data),
            "Check the return value of `erc1155BatchWithdrawal`"
        ).to.equal(true);

        // check emitted event
        expect(
            await debugFacet._erc1155BatchWithdrawal(data),
            "Check if `ERC1155BatchWithdrawn` is emmitted"
        )
            .to.emit(portalFacet, "ERC1155BatchWithdrawn")
            .withArgs(erc1155, receiver, tokenIds, tokenAmounts, transferData);
    });
});
