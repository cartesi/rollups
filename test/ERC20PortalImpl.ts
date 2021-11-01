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
import { solidity, MockProvider, deployContract } from "ethereum-waffle";
import { ERC20PortalImpl__factory } from "../src/types/factories/ERC20PortalImpl__factory";
import { Signer } from "ethers";
import { ERC20PortalImpl } from "../src/types/ERC20PortalImpl";
import { keccak256 } from "ethers/lib/utils";
import { IERC20 } from "../src/types/IERC20";

use(solidity);

describe("ERC20Portal Implementation", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalImpl: ERC20PortalImpl;
    let mockInput: MockContract; //mock input
    let mockERC20: MockContract; //mock erc20
    let portalFactory: ERC20PortalImpl__factory;

    beforeEach(async () => {
        await deployments.fixture();
        [signer, signer2] = await ethers.getSigners();

        const Input = await deployments.getArtifact("InputImpl");
        const CTSI = await deployments.getArtifact("IERC20");

        mockInput = await deployMockContract(signer, Input.abi);
        mockERC20 = await deployMockContract(signer, CTSI.abi);

        portalFactory = new ERC20PortalImpl__factory(signer);

        portalImpl = await portalFactory.deploy(
            mockInput.address,
            await signer.getAddress()
        );
    });

    it("erc20Deposit should revert if transfer from returns false", async () => {
        await mockERC20.mock.transferFrom.returns(false);

        await expect(
            portalImpl.erc20Deposit(
                mockERC20.address,
                50,
                "0x00"
            ),
            "ether deposit should revert if ERC20 transferFrom fails"
        ).to.be.revertedWith("ERC20 transferFrom failed");
    });

    it("erc20Deposit should emit events", async () => {
        await mockERC20.mock.transferFrom.returns(true);
        let B32str = keccak256("0x00");
        await mockInput.mock.addInput.returns(B32str);

        expect(
            await portalImpl.erc20Deposit(
                mockERC20.address,
                15,
                "0x00"
            ),
            "expect erc20Deposit function to emit ERC20Deposited event"
        )
            .to.emit(portalImpl, "ERC20Deposited")
            .withArgs(
                mockERC20.address,
                await signer.getAddress(),
                15,
                "0x00"
            );

        expect(
            await portalImpl.erc20Deposit(
                mockERC20.address,
                1000000,
                "0x00"
            ),
            "expect erc20Deposit function to emit ERC20Deposited event"
        )
            .to.emit(portalImpl, "ERC20Deposited")
            .withArgs(
                mockERC20.address,
                await signer.getAddress(),
                1000000,
                "0x00"
            );
    });

    it("erc20Deposit should return the return value of inputContract.addInput()", async () => {
        await mockERC20.mock.transferFrom.returns(true);
        let B32str = keccak256("0x00");
        await mockInput.mock.addInput.returns(B32str);

        expect(
            await portalImpl.callStatic.erc20Deposit(
                mockERC20.address,
                15,
                "0x00"
            ),
            "callStatic to check return value"
        ).to.equal(B32str);
    });

    it("executeDescartesV2Output should revert if not called from output", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                mockERC20.address,
                await signer.getAddress(),
                10,
            ]
        );
        await expect(
            portalImpl.connect(signer2).executeDescartesV2Output(data)
        ).to.be.revertedWith("only outputContract");
    });

    it("executeDescartesV2Output should emit ERC20Withdrawn and return true", async () => {
        await mockERC20.mock.transfer.returns(true);

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                mockERC20.address,
                await signer.getAddress(),
                10,
            ]
        );

        // callStatic check return value
        expect(
            await portalImpl.callStatic.executeDescartesV2Output(data)
        ).to.equal(true);
        // check emitted event
        await expect(portalImpl.executeDescartesV2Output(data))
            .to.emit(portalImpl, "ERC20Withdrawn")
            .withArgs(mockERC20.address, await signer.getAddress(), 10);
    });
});
