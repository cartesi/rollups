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
import { PortalImpl__factory } from "../src/types/factories/PortalImpl__factory";
import { IERC20__factory } from "../src/types/factories/IERC20__factory";
import { Signer } from "ethers";
import { PortalImpl } from "../src/types/PortalImpl";
import { keccak256 } from "ethers/lib/utils";
import { IERC20 } from "../src/types/IERC20";

use(solidity);

describe("Portal Implementation", async () => {
    let signer: Signer;
    var portalImpl: PortalImpl;
    let mockInput: MockContract; //mock input
    let mockERC20: MockContract; //mock erc20
    let portalFactory: PortalImpl__factory;

    beforeEach(async () => {
        [signer] = await ethers.getSigners();

        const Input = await deployments.getArtifact("InputImpl");
        const CTSI = await deployments.getArtifact("IERC20");

        mockInput = await deployMockContract(signer, Input.abi);
        mockERC20 = await deployMockContract(signer, CTSI.abi);

        portalFactory = new PortalImpl__factory(signer);

        portalImpl = await portalFactory.deploy(
            mockInput.address,
            await signer.getAddress()
        );
    });

    it("etherDeposit should revert if parameters are inconsistent", async () => {
        expect(
            portalImpl.etherDeposit(
                [await signer.getAddress()],
                [50, 30],
                "0x00",
                {
                    value: ethers.utils.parseEther("50"),
                }
            ),
            "ether deposit should revert if amount.length > addresses.length"
        ).to.be.revertedWith("receivers array length != amounts array length");

        expect(
            portalImpl.etherDeposit(
                [await signer.getAddress(), mockInput.address],
                [50],
                "0x00",
                { value: ethers.utils.parseEther("50") }
            ),
            "ether deposit should revert if amount.length > addresses.length"
        ).to.be.revertedWith("receivers array length != amounts array length");
    });

    it("etherDeposit should revert if msg.value is too low", async () => {
        expect(
            portalImpl.etherDeposit(
                [await signer.getAddress(), mockInput.address],
                [
                    ethers.utils.parseEther("100"),
                    ethers.utils.parseEther("100"),
                ],
                "0x00",
                { value: ethers.utils.parseEther("100") }
            ),
            "ether deposit should revert if not enough ether was sent"
        ).to.be.revertedWith("msg.value < totalAmount");

        expect(
            portalImpl.etherDeposit(
                [await signer.getAddress(), mockInput.address],
                [
                    ethers.utils.parseEther("100"),
                    ethers.utils.parseEther("100"),
                ],
                "0x00",
                { value: ethers.utils.parseEther("199") }
            ),
            "ether deposit should revert if not enough ether was sent"
        ).to.be.revertedWith("msg.value < totalAmount");
    });

    it("etherDeposit should emit events", async () => {
        await mockInput.mock.addInput.returns(keccak256("0x00"));

        expect(
            await portalImpl.etherDeposit(
                [await signer.getAddress()],
                [50],
                "0x00",
                {
                    value: ethers.utils.parseEther("50"),
                }
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        ).to.emit(portalImpl, "EtherDeposited");

        expect(
            await portalImpl.etherDeposit(
                [await signer.getAddress(), mockInput.address],
                [15, 45],
                "0x00",
                { value: ethers.utils.parseEther("60") }
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        ).to.emit(portalImpl, "EtherDeposited");

        expect(
            await portalImpl.etherDeposit(
                [
                    await signer.getAddress(),
                    mockInput.address,
                    mockInput.address,
                ],
                [15, 45, 30],
                "0x00",
                { value: ethers.utils.parseEther("90") }
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        ).to.emit(portalImpl, "EtherDeposited");
    });

    it("erc20Deposit should revert if parameters are inconsistent", async () => {
        await expect(
            portalImpl.erc20Deposit(
                mockERC20.address,
                mockERC20.address,
                [await signer.getAddress()],
                [50, 30],
                "0x00"
            ),
            "ether deposit should revert if amount.length > addresses.length"
        ).to.be.revertedWith("receivers array length != amounts array length");

        await expect(
            portalImpl.erc20Deposit(
                mockERC20.address,
                mockERC20.address,
                [await signer.getAddress()],
                [],
                "0x00"
            ),
            "ether deposit should revert if amount.length > addresses.length"
        ).to.be.revertedWith("receivers array length != amounts array length");
    });

    it("erc20Deposit should revert if transfer from returns false", async () => {
        await mockERC20.mock.transferFrom.returns(false);

        await expect(
            portalImpl.erc20Deposit(
                mockERC20.address,
                mockERC20.address,
                [await signer.getAddress()],
                [50],
                "0x00"
            ),
            "ether deposit should revert if amount.length > addresses.length"
        ).to.be.revertedWith("erc20 transferFrom failed");
    });

    it("erc20Deposit should emit events", async () => {
        await mockERC20.mock.transferFrom.returns(true);
        await mockInput.mock.addInput.returns(keccak256("0x00"));

        expect(
            await portalImpl.erc20Deposit(
                mockERC20.address,
                await signer.getAddress(),
                [await signer.getAddress(), mockInput.address],
                [15, 45],
                "0x00"
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        ).to.emit(portalImpl, "ERC20Deposited");

        expect(
            await portalImpl.erc20Deposit(
                mockERC20.address,
                await signer.getAddress(),
                [
                    await signer.getAddress(),
                    mockInput.address,
                    mockInput.address,
                ],
                [1000000, 15, 45],
                "0x00"
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        ).to.emit(portalImpl, "ERC20Deposited");
    });
});
