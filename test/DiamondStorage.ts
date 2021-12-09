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
import { Signer, BigNumber } from "ethers";
import { DebugFacet } from "../dist/src/types/DebugFacet";
import { DebugFacet__factory } from "../dist/src/types/factories/DebugFacet__factory";

use(solidity);

describe("Diamond Storage", async () => {
    let signer: Signer;
    var debugFacet: DebugFacet;

    beforeEach(async () => {
        await deployments.fixture(["DebugDiamond"]);
        [signer] = await ethers.getSigners();
        const diamondAddress = (await deployments.get("CartesiRollupsDebug")).address;
        debugFacet = DebugFacet__factory.connect(diamondAddress, signer);
    });

    it("check for collisions in Diamond Storages", async () => {
        const n = await debugFacet._getDiamondStoragePositions();
        let positions: BigNumber[] = [];
        for (let i = BigNumber.from(0); i.lt(n); i = i.add(1)) {
            let pos = await debugFacet._getDiamondStoragePosition(i);
            positions.push(BigNumber.from(pos)); // add positions from all Diamond Storages
        }
        positions.sort((a, b) => {
            if (a.gt(b))      return 1;
            else if (a.lt(b)) return -1;
            else              return 0;
        }); // make nearby positions adjacent
        for (let i = 1; i < positions.length; i++) {
            let dist = positions[i].sub(positions[i-1]);
            expect(
                dist.gt(256), // 256 slots
                "Risk of collision detected"
            ).to.be.true;
        }
    });
});
