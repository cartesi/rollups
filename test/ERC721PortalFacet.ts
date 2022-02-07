// Copyright (C) 2022 Cartesi Pte. Ltd.

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
import { ERC721PortalFacet } from "../dist/src/types/ERC721PortalFacet";
import { ERC721PortalFacet__factory } from "../dist/src/types/factories/ERC721PortalFacet__factory";
import { DebugFacet } from "../dist/src/types/DebugFacet";
import { DebugFacet__factory } from "../dist/src/types/factories/DebugFacet__factory";
import { IERC721 } from "../dist/src/types/IERC721";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import { getInputHash } from "./utils";

use(solidity);

describe("ERC721Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC721PortalFacet;
    var debugFacet: DebugFacet;
    let mockERC721: MockContract; //mock erc721

    beforeEach(async () => {
        await deployments.fixture(["DebugDiamond"]);
        [signer, signer2] = await ethers.getSigners();
        const diamondAddress = (await deployments.get("CartesiRollupsDebug")).address;
        portalFacet = ERC721PortalFacet__factory.connect(diamondAddress, signer);
        debugFacet = DebugFacet__factory.connect(diamondAddress, signer);

        // Deploy a mock ERC-721 contract
        const IERC721 = await deployments.getArtifact("IERC721");
        mockERC721 = await deployMockContract(signer, IERC721.abi);
    });

    it("erc721Deposit should revert if safeTransferFrom reverts", async () => {
        const reason = "This cryptokitty is not available";
        await mockERC721.mock['safeTransferFrom(address,address,uint256)'].revertsWithReason(reason);

        const erc721 = mockERC721.address;
        const tokenId = 50;
        const data = "0x00";

        await expect(
            portalFacet.erc721Deposit(
                erc721,
                tokenId,
                data
            ),
            "ERC721 deposit should revert if ERC721 safeTransferFrom fails"
        ).to.be.revertedWith(reason);
    });

    it("erc721Deposit should emit events", async () => {
        await mockERC721.mock['safeTransferFrom(address,address,uint256)'].returns();

        const erc721 = mockERC721.address;
        const sender = await signer.getAddress();
        const tokenId = 15;
        const data = "0x00";

        expect(
            await portalFacet.erc721Deposit(erc721, tokenId, data),
            "expect erc721Deposit function to emit ERC721Deposited event"
        )
            .to.emit(portalFacet, "ERC721Deposited")
            .withArgs(erc721, sender, tokenId, data);
    });

    it("erc721Deposit should return LibInput.addInput(...)", async () => {
        await mockERC721.mock['safeTransferFrom(address,address,uint256)'].returns();

        const header = keccak256(toUtf8Bytes("ERC721_Transfer"));
        const erc721 = mockERC721.address;
        const sender = await signer.getAddress();
        const tokenId = 15;
        const data = "0x00";

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            ["bytes32", "address", "address", "uint", "bytes"],
            [header, sender, erc721, tokenId, data]
        );

        // Calculate the input hash
        const block = await ethers.provider.getBlock("latest");
        const inputHash = getInputHash(input, sender, block.number, block.timestamp, 0x0, 0x0);

        expect(
            await portalFacet.callStatic.erc721Deposit(erc721, tokenId, data),
            "callStatic to check return value"
        ).to.equal(inputHash);
    });

    it("erc721Withdrawal should revert if not called by the Rollups contract", async () => {
        const erc721 = mockERC721.address;
        const receiver = await signer.getAddress();
        const tokenId = 15;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                erc721,
                receiver,
                tokenId,
            ]
        );
        await expect(
            portalFacet.connect(signer2).erc721Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("erc721Withdrawal should emit ERC721Withdrawn and return true", async () => {
        await mockERC721.mock['safeTransferFrom(address,address,uint256)'].returns();

        const erc721 = mockERC721.address;
        const receiver = await signer.getAddress();
        const tokenId = 42;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [erc721, receiver, tokenId]
        );

        // callStatic check return value
        expect(
            await debugFacet.callStatic._erc721Withdrawal(data)
        ).to.equal(true);

        // check emitted event
        await expect(debugFacet._erc721Withdrawal(data))
            .to.emit(portalFacet, "ERC721Withdrawn")
            .withArgs(erc721, receiver, tokenId);
    });
});
