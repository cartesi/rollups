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

// Note: This component currently has dependencies that are licensed under the GU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { MockProvider } from "ethereum-waffle";
import { ethers } from "hardhat";
import { SimpleToken__factory } from "../dist/src/types/factories/SimpleToken__factory";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments } = hre;
    const { diamond } = deployments;
    const MINUTE = 60; // seconds in a minute
    const HOUR = 60 * MINUTE; // seconds in an hour
    const DAY = 24 * HOUR; // seconds in a day

    const INPUT_DURATION = 1 * DAY;
    const CHALLENGE_PERIOD = 7 * DAY;
    const INPUT_LOG2_SIZE = 8;
    const CTSI_ADDRESS = "0x491604c0FDF08347Dd1fa4Ee062a822A5DD06B5D";

    let signers = await ethers.getSigners();

    const provider = new MockProvider();
    const wallets = provider.getWallets();
    var validators: string[] = [];

    // add up to 8 wallets as validators
    for (var wallet of wallets) {
        let address = await wallet.getAddress();
        validators.push(address);
        if (validators.length == 8) break;
    }

    // Bitmask
    const bitMaskLibrary = await deployments.deploy("Bitmask", {
        from: await signers[0].getAddress(),
    });
    const bitMaskAddress = bitMaskLibrary.address;

    // CartesiMath
    const cartesiMath = await deployments.deploy("CartesiMath", {
        from: await signers[0].getAddress(),
    });
    const cartesiMathAddress = cartesiMath.address;

    // Merkle
    const merkle = await deployments.deploy("Merkle", {
        from: await signers[0].getAddress(),
        libraries: {
            CartesiMath: cartesiMathAddress,
        },
    });
    const merkleAddress = merkle.address;

    // ClaimsMaskLibrary
    const claimsMaskLibrary = await deployments.deploy("ClaimsMaskLibrary", {
        from: await signers[0].getAddress(),
    });
    const claimsMaskLibraryAddress = claimsMaskLibrary.address;

    // Simple ERC20 token
    let tokenSupply = 1000000; // assume FeeManagerImpl contract owner has 1 million tokens (ignore decimals)
    let initialFeePerClaim = 10; // set initial fees per claim as 10 token
    let deployedToken = await deployments.deploy("SimpleToken", {
        from: await signers[0].getAddress(),
        args: [tokenSupply],
    });
    let token = SimpleToken__factory.connect(deployedToken.address, signers[0]);

    // CartesiRollups
    const { address } = await diamond.deploy("CartesiRollupsDebug", {
        from: await signers[0].getAddress(),
        owner: await signers[0].getAddress(),
        facets: [
            'InputFacet',
            'RollupsFacet',
            'RollupsInitFacet',
            'ValidatorManagerFacet',
            'OutputFacet',
            'EtherPortalFacet',
            'ERC20PortalFacet',
            'SERC20PortalFacet',
            'ERC721PortalFacet',
            'FeeManagerFacet',
            'DebugFacet', // For debug pursposes only
        ],
        libraries: {
            ClaimsMaskLibrary: claimsMaskLibraryAddress,
            Bitmask: bitMaskAddress,
            Merkle: merkleAddress,
        },
        execute : {
            methodName: 'init',
            args: [
                INPUT_DURATION,
                CHALLENGE_PERIOD,
                INPUT_LOG2_SIZE,
                initialFeePerClaim,
                validators,
                CTSI_ADDRESS,
                token,
            ],
        },
    });

    console.log("Debug Diamond address: " + address);
};

export default func;
func.tags = ["DebugDiamond"];
