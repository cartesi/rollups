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
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { EtherPortalFacet } from "../dist/src/types/EtherPortalFacet";
import { EtherPortalFacet__factory } from "../dist/src/types/factories/EtherPortalFacet__factory";
import { DebugFacet } from "../dist/src/types/DebugFacet";
import { DebugFacet__factory } from "../dist/src/types/factories/DebugFacet__factory";
import { keccak256 } from "ethers/lib/utils";

use(solidity);

describe("EtherPortal Facet", async () => {
    let signer: Signer;
    let signer2: Signer;
    var portalFacet: EtherPortalFacet;
    var debugFacet: DebugFacet;

    beforeEach(async () => {
        await deployments.fixture(["DebugDiamond"]);
        [signer, signer2] = await ethers.getSigners();
        const diamondAddress = (await deployments.get("CartesiRollupsDebug")).address;
        portalFacet = EtherPortalFacet__factory.connect(diamondAddress, signer);
        debugFacet = DebugFacet__factory.connect(diamondAddress, signer);
    });

    it("etherDeposit should emit events", async () => {
        const data = "0x00";
        const value = ethers.utils.parseEther("60");
        
        expect(
            await portalFacet.etherDeposit(data, {value: value}),
            "expect etherDeposit function to emit EtherDeposited event"
        )
            .to.emit(portalFacet, "EtherDeposited")
            .withArgs(await signer.getAddress(), value, data);
    });

    it("etherWithdrawal should revert if not called by the Diamond itself", async () => {
        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [
                await signer.getAddress(),
                10,
            ]
        );
        await expect(
            portalFacet.connect(signer2).etherWithdrawal(data)
        ).to.be.revertedWith("only itself");
    });

    it("etherWithdrawal should emit EtherWithdrawn and return true", async () => {
        // deposit ethers to portalFacet for enough balance to a successful transfer in etherWithdrawal()
        await portalFacet.etherDeposit(
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
            await debugFacet.callStatic._etherWithdrawal(data)
        ).to.equal(true);

        // check emitted event
        await expect(debugFacet._etherWithdrawal(data))
            .to.emit(portalFacet, "EtherWithdrawn")
            .withArgs(await signer.getAddress(), 10);
    });

    it("etherDeposit should return the return value of LibInput.addInput()", async () => {
        // create some random data
        let sender = await signer.getAddress();
        let value = ethers.utils.parseEther("10");
        let data = "0xdeadbeef";
        let timestamp = (await ethers.provider.getBlock("latest")).timestamp;

        // ABI encode the input
        let input = ethers.utils.defaultAbiCoder.encode(
            ["address", "uint", "bytes"],
            [
                sender, // msg.sender
                value,  // msg.value
                data,   // _data
            ]
        );

        // calculate input hash: keccak256(abi.encode(keccak256(metadata), keccak256(_input)))
        // metadata: abi.encode(msg.sender, block.timestamp)
        let metadata = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [
                sender,    // msg.sender
                timestamp, // block.timestamp
            ]
        );
        let keccak_metadata = ethers.utils.keccak256(metadata);
        let keccak_input = ethers.utils.keccak256(input);
        let abi_metadata_input = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint"],
            [keccak_metadata, keccak_input]
        );
        let input_hash = ethers.utils.keccak256(abi_metadata_input);

        // check if input hashes are identical
        expect(
            await portalFacet.callStatic.etherDeposit(data, {value: value}),
            "callStatic to check return value"
        ).to.equal(input_hash);
    });

});
