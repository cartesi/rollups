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
import { EtherPortalImpl__factory } from "../src/types/factories/EtherPortalImpl__factory";
import { Signer } from "ethers";
import { EtherPortalImpl } from "../src/types/EtherPortalImpl";
import { keccak256 } from "ethers/lib/utils";

use(solidity);

describe("EtherPortal Implementation", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalImpl: EtherPortalImpl;
    let mockInput: MockContract; //mock input
    let portalFactory: EtherPortalImpl__factory;

    beforeEach(async () => {
        await deployments.fixture();
        [signer, signer2] = await ethers.getSigners();

        const Input = await deployments.getArtifact("InputImpl");

        mockInput = await deployMockContract(signer, Input.abi);

        portalFactory = new EtherPortalImpl__factory(signer);

        portalImpl = await portalFactory.deploy(
            mockInput.address,
            await signer.getAddress()
        );
    });

    it("etherDeposit should emit events", async () => {
        await mockInput.mock.addInput.returns(keccak256("0x00"));

        expect(
            await portalImpl.etherDeposit(
                "0x00",
                { value: ethers.utils.parseEther("60") }
            ),
            "expect etherDeposit function to emit EtherDeposited event"
        )
            .to.emit(portalImpl, "EtherDeposited")
            .withArgs(
                await signer.getAddress(),
                ethers.utils.parseEther("60"),
                "0x00"
            );

    });

    it("executeDescartesV2Output should revert if not called from output", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [
                await signer.getAddress(),
                10,
            ]
        );
        await expect(
            portalImpl.connect(signer2).executeDescartesV2Output(data)
        ).to.be.revertedWith("only outputContract");
    });

    it("executeDescartesV2Output should emit EtherWithdrawn and return true", async () => {
        // deposit ethers to portalImpl for enough balance to call function transfer() in etherWithdrawal()
        await mockInput.mock.addInput.returns(keccak256("0x00"));
        await portalImpl.etherDeposit(
            "0x00",
            {
                value: ethers.utils.parseEther("10"),
            }
        );

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [
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
            .to.emit(portalImpl, "EtherWithdrawn")
            .withArgs(await signer.getAddress(), 10);
    });

    it("etherDeposit should return the return value of inputContract.addInput()", async () => {
        let B32str = ethers.utils.formatBytes32String("hello");
        await mockInput.mock.addInput.returns(B32str);

        expect(
            await portalImpl.callStatic.etherDeposit(
                "0x00",
                {
                    value: ethers.utils.parseEther("50"),
                }
            ),
            "callStatic to check return value"
        ).to.equal(B32str);
    });

});
