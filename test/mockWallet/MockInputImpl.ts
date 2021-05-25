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
import { solidity } from "ethereum-waffle";
import { MockInputImpl__factory } from "../../src/types/factories/MockInputImpl__factory";
import { Signer } from "ethers";
import { MockInputImpl } from "../../src/types/MockInputImpl";

use(solidity);

const OPERATION = {
    EtherOp: 0,
    ERC20Op: 1
};

const TRANSACTION = {
    Deposit: 0,
    Transfer: 1,
    Withdraw: 2
};

describe("Mock Input Implementation", () => {
    let signer: Signer;
    let inputImpl: MockInputImpl;
    let mockDescartesv2: MockContract; //mock descartesv2 implementation

    beforeEach(async () => {
        [signer] = await ethers.getSigners();

        const DescartesV2 = await deployments.getArtifact("DescartesV2");

        mockDescartesv2 = await deployMockContract(signer, DescartesV2.abi);

        const inputFactory = new MockInputImpl__factory(signer);

        inputImpl = await inputFactory.deploy(
            mockDescartesv2.address
        );
    });

    it("addInput should revert if input length == 0", async () => {
        await expect(
            inputImpl.addInput([],0 ),
            "empty input should revert"
        ).to.be.revertedWith("input length should be greater than 0");
    });

    it("addInput should add input to inbox", async () => {
        const encodedInput = ethers.utils.defaultAbiCoder.encode(
            [ "uint", "uint", "address[]" ,"uint256[]"],
            [ 0, 0, ["0xC2d5eBeDe07e556266eba180F537a28EC46b992e"], [50] ]
        );

        await inputImpl.addInput(encodedInput, 0);
        await inputImpl.addInput(encodedInput, 0);

        expect(
            await inputImpl.getNumberOfInputs(),
            "Number of inputs should be 2"
        ).to.equal(2);
    });

    it("emit events when input is added", async () => {
        const encodedInput = ethers.utils.defaultAbiCoder.encode(
            [ "uint", "uint", "address[]" ,"uint256[]"],
            [ 0, 0, ["0xC2d5eBeDe07e556266eba180F537a28EC46b992e"], [50] ]
        );

        expect(
            await inputImpl.addInput(encodedInput, 0),
            "expect addInput function to emit EtherInputAdded event"
        ).to.emit(inputImpl, "EtherInputAdded");

        const eventFilter = inputImpl.filters.EtherInputAdded(
            null,
            null,
            null,
            null
        );

        const event = await inputImpl.queryFilter(eventFilter);
        let eventArgs = event[0]["args"];

        expect(eventArgs["_operation"], "input operation should be Eth").to.equal(
            OPERATION.EtherOp
        );

        expect(eventArgs["_transaction"], "input transaction should be Deposit").to.equal(
            TRANSACTION.Deposit
        );

        expect(eventArgs["_receivers"], "input receivers should be an array of receivers").to.contains(
            "0xC2d5eBeDe07e556266eba180F537a28EC46b992e"
        );

        expect(parseInt(eventArgs["_amounts"].toString()), "input amount should be present").to.equal(
            50
        );
    });

    it("addInput should revert if it input box size is more than 10", async () => {
        const encodedInput = ethers.utils.defaultAbiCoder.encode(
            [ "uint", "uint", "address[]" ,"uint256[]" , "bytes"],
            [ 0, 0, ["0xC2d5eBeDe07e556266eba180F537a28EC46b992e"], [50], "0x00" ]
        );
        expect(
            await inputImpl.getNumberOfInputs(),
            "current inbox should start as zero"
        ).to.equal(0);

        for (let i = 0; i < 10; i++){
            await inputImpl.addInput(encodedInput, 0);
        }

        await expect(
            inputImpl.addInput(encodedInput, 0),
            "input can't be bigger than 10"
        ).to.be.revertedWith("input box size cannot be greater than 10");
    });
});
