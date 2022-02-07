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
import { ERC20PortalFacet } from "../dist/src/types/ERC20PortalFacet";
import { ERC20PortalFacet__factory } from "../dist/src/types/factories/ERC20PortalFacet__factory";
import { DebugFacet } from "../dist/src/types/DebugFacet";
import { DebugFacet__factory } from "../dist/src/types/factories/DebugFacet__factory";
import { IERC20 } from "../dist/src/types/IERC20";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import { getInputHash } from "./utils";

use(solidity);

describe("ERC20Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: ERC20PortalFacet;
    var debugFacet: DebugFacet;
    let mockERC20: MockContract; //mock erc20

    beforeEach(async () => {
        await deployments.fixture(["DebugDiamond"]);
        [signer, signer2] = await ethers.getSigners();
        const diamondAddress = (await deployments.get("CartesiRollupsDebug")).address;
        portalFacet = ERC20PortalFacet__factory.connect(diamondAddress, signer);
        debugFacet = DebugFacet__factory.connect(diamondAddress, signer);

        // Deploy a mock ERC-20 contract
        const CTSI = await deployments.getArtifact("IERC20");
        mockERC20 = await deployMockContract(signer, CTSI.abi);
    });

    it("erc20Deposit should revert if transferFrom returns false", async () => {
        await mockERC20.mock.transferFrom.returns(false);

        await expect(
            portalFacet.erc20Deposit(
                mockERC20.address,
                50,
                "0x00"
            ),
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
        const inputHash = getInputHash(input, sender, block.number, block.timestamp, 0x0, 0x0);

        expect(
            await portalFacet.callStatic.erc20Deposit(erc20, value, data),
            "callStatic to check return value"
        ).to.equal(inputHash);
    });

    it("erc20Withdrawal should revert if not called by the Rollups contract", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                mockERC20.address,
                await signer.getAddress(),
                10,
            ]
        );
        await expect(
            portalFacet.connect(signer2).erc20Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("erc20Withdrawal should emit ERC20Withdrawn and return true", async () => {
        await mockERC20.mock.transfer.returns(true);

        const erc20 = mockERC20.address;
        const sender = await signer.getAddress();
        const value = 42;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [erc20, sender, value]
        );

        // callStatic check return value
        expect(
            await debugFacet.callStatic._erc20Withdrawal(data)
        ).to.equal(true);

        // check emitted event
        await expect(debugFacet._erc20Withdrawal(data))
            .to.emit(portalFacet, "ERC20Withdrawn")
            .withArgs(erc20, sender, value);
    });
});
