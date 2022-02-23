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
import { SERC20PortalFacet } from "../src/types/SERC20PortalFacet";
import { SERC20PortalFacet__factory } from "../src/types/factories/SERC20PortalFacet__factory";
import { DebugFacet } from "../src/types/DebugFacet";
import { DebugFacet__factory } from "../src/types/factories/DebugFacet__factory";
import { IERC20 } from "../src/types/IERC20";
import { keccak256, toUtf8Bytes } from "ethers/lib/utils";
import { deployDiamond, getInputHash } from "./utils";

use(solidity);

describe("SERC20Portal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: SERC20PortalFacet;
    var debugFacet: DebugFacet;
    let mockERC20: MockContract; //mock erc20

    beforeEach(async () => {
        [signer, signer2] = await ethers.getSigners();

        // Note: we could pass a mockERC20 here to `erc20ForPortal` but that
        // would make a new fixture for every test case, which is not really
        // needed. Instead, we inject the address of the mock contract via
        // the `_setSERC20Address` function of the Debug facet.
        const diamond = await deployDiamond({ debug: true });
        portalFacet = SERC20PortalFacet__factory.connect(
            diamond.address,
            signer
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signer);

        // Deploy a mock ERC-20 contract
        const CTSI = await deployments.getArtifact("IERC20");
        mockERC20 = await deployMockContract(signer, CTSI.abi);

        // Inject mock into specific ERC-20 portal diamond storage
        await debugFacet._setSERC20Address(mockERC20.address);
    });

    it("serc20Deposit should revert if transferFrom returns false", async () => {
        await mockERC20.mock.transferFrom.returns(false);

        await expect(
            portalFacet.serc20Deposit(50, "0x00"),
            "Specific ERC20 deposit should revert if ERC20 transferFrom fails"
        ).to.be.revertedWith("ERC20 transferFrom failed");
    });

    it("serc20Deposit should emit events", async () => {
        await mockERC20.mock.transferFrom.returns(true);

        const sender = await signer.getAddress();
        const value = 15;
        const data = "0x00";

        expect(
            await portalFacet.serc20Deposit(value, data),
            "expect serc20Deposit function to emit SERC20Deposited event"
        )
            .to.emit(portalFacet, "SERC20Deposited")
            .withArgs(sender, value, data);
    });

    it("serc20Deposit should return the return value of LibInput.addInput()", async () => {
        await mockERC20.mock.transferFrom.returns(true);

        const header = keccak256(toUtf8Bytes("Specific_ERC20_Transfer"));
        const sender = await signer.getAddress();
        const value = 15;
        const data = "0x00";

        // Encode input using the default ABI
        const input = ethers.utils.defaultAbiCoder.encode(
            ["bytes32", "address", "uint", "bytes"],
            [header, sender, value, data]
        );

        // Calculate the input hash
        const block = await ethers.provider.getBlock("latest");
        const inputHash = getInputHash(
            input,
            portalFacet.address,
            block.number,
            block.timestamp,
            0x0,
            0x0
        );

        expect(
            await portalFacet.callStatic.serc20Deposit(value, data),
            "callStatic to check return value"
        ).to.equal(inputHash);
    });

    it("serc20Withdrawal should revert if not called by the Rollups contract", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [await signer.getAddress(), 10]
        );
        await expect(
            portalFacet.connect(signer2).serc20Withdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("serc20Withdrawal should emit SERC20Withdrawn and return true", async () => {
        await mockERC20.mock.transfer.returns(true);

        const sender = await signer.getAddress();
        const value = 42;

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [sender, value]
        );

        // callStatic check return value
        expect(await debugFacet.callStatic._serc20Withdrawal(data)).to.equal(
            true
        );

        // check emitted event
        await expect(debugFacet._serc20Withdrawal(data))
            .to.emit(portalFacet, "SERC20Withdrawn")
            .withArgs(sender, value);
    });
});
