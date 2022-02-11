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
import { ethers } from "hardhat";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments } = hre;

    const signers = await ethers.getSigners();
    const contractOwnerAddress = await signers[0].getAddress();

    // Diamond libraries
    // =================

    console.log();
    console.log("===> Deploying essential diamond libraries");

    // LibDiamond
    const libDiamond = await deployments.deploy("LibDiamond", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libDiamond.address }] Deployed LibDiamond`);

    // Diamond facets
    // ==============

    console.log();
    console.log("===> Deploying essential diamond facets");

    // DiamondCutFacet
    const diamondCutFacet = await deployments.deploy("DiamondCutFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ diamondCutFacet.address }] Deployed DiamondCutFacet`);

    // DiamondLoupeFacet
    const diamondLoupeFacet = await deployments.deploy("DiamondLoupeFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ diamondLoupeFacet.address }] Deployed DiamondLoupeFacet`);

    // OwnershipFacet
    const ownershipFacet = await deployments.deploy("OwnershipFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ ownershipFacet.address }] Deployed OwnershipFacet`);

    // Utility libraries
    // =================

    console.log();
    console.log("===> Deploying utility libraries");

    // Bitmask
    const bitmask = await deployments.deploy("Bitmask", {
        from: contractOwnerAddress,
    });
    console.log(`[${ bitmask.address }] Deployed Bitmask`);

    // CartesiMath
    const cartesiMath = await deployments.deploy("CartesiMath", {
        from: contractOwnerAddress,
    });
    console.log(`[${ cartesiMath.address }] Deployed CartesiMath`);

    // Merkle
    const merkle = await deployments.deploy("Merkle", {
        from: contractOwnerAddress,
        libraries: {
            CartesiMath: cartesiMath.address,
        },
    });
    console.log(`[${ merkle.address }] Deployed Merkle`);

    // LibClaimsMask
    const libClaimsMask = await deployments.deploy("LibClaimsMask", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libClaimsMask.address }] Deployed LibClaimsMask`);

    // Rollups libraries
    // =================

    console.log();
    console.log("===> Deploying rollups libraries");

    // LibInput
    const libInput = await deployments.deploy("LibInput", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libInput.address }] Deployed LibInput`);

    // LibOutput
    const libOutput = await deployments.deploy("LibOutput", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libOutput.address }] Deployed LibOutput`);

    // LibValidatorManager
    const libValidatorManager = await deployments.deploy("LibValidatorManager", {
        from: contractOwnerAddress,
        libraries: {
            LibClaimsMask: libClaimsMask.address,
        },
    });
    console.log(`[${ libValidatorManager.address }] Deployed LibValidatorManager`);

    // LibDisputeManager
    const libDisputeManager = await deployments.deploy("LibDisputeManager", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libDisputeManager.address }] Deployed LibDisputeManager`);

    // LibSERC20Portal
    const libSERC20Portal = await deployments.deploy("LibSERC20Portal", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libSERC20Portal.address }] Deployed LibSERC20Portal`);

    // LibFeeManager
    const libFeeManager = await deployments.deploy("LibFeeManager", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libFeeManager.address }] Deployed LibFeeManager`);

    // LibRollups
    const libRollups = await deployments.deploy("LibRollups", {
        from: contractOwnerAddress,
    });
    console.log(`[${ libRollups.address }] Deployed LibRollups`);

    // Rollups facets
    // ==============

    console.log();
    console.log("===> Deploying rollups facets");

    // ERC20PortalFacet
    const erc20PortalFacet = await deployments.deploy("ERC20PortalFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ erc20PortalFacet.address }] Deployed ERC20PortalFacet`);

    // ERC721PortalFacet
    const erc721PortalFacet = await deployments.deploy("ERC721PortalFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ erc721PortalFacet.address }] Deployed ERC721PortalFacet`);

    // EtherPortalFacet
    const etherPortalFacet = await deployments.deploy("EtherPortalFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ etherPortalFacet.address }] Deployed EtherPortalFacet`);

    // FeeManagerFacet
    const feeManagerFacet = await deployments.deploy("FeeManagerFacet", {
        from: contractOwnerAddress,
        libraries: {
            LibClaimsMask: libClaimsMask.address,
        },
    });
    console.log(`[${ feeManagerFacet.address }] Deployed FeeManagerFacet`);

    // InputFacet
    const inputFacet = await deployments.deploy("InputFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ inputFacet.address }] Deployed InputFacet`);

    // OutputFacet
    const outputFacet = await deployments.deploy("OutputFacet", {
        from: contractOwnerAddress,
        libraries: {
            Bitmask: bitmask.address,
            Merkle: merkle.address,
        },
    });
    console.log(`[${ outputFacet.address }] Deployed OutputFacet`);

    // RollupsFacet
    const rollupsFacet = await deployments.deploy("RollupsFacet", {
        from: contractOwnerAddress,
        libraries: {
            LibClaimsMask: libClaimsMask.address,
        },
    });
    console.log(`[${ rollupsFacet.address }] Deployed RollupsFacet`);

    // SERC20PortalFacet
    const serc20PortalFacet = await deployments.deploy("SERC20PortalFacet", {
        from: contractOwnerAddress,
    });
    console.log(`[${ serc20PortalFacet.address }] Deployed SERC20PortalFacet`);

    // ValidatorManagerFacet
    const validatorManagerFacet = await deployments.deploy("ValidatorManagerFacet", {
        from: contractOwnerAddress,
        libraries: {
            LibClaimsMask: libClaimsMask.address,
        },
    });
    console.log(`[${ validatorManagerFacet.address }] Deployed ValidatorManagerFacet`);

    // Diamond initialization contract
    // ===============================

    console.log();
    console.log("===> Deploying initialization contracts");

    // DiamondInit
    const diamondInit = await deployments.deploy("DiamondInit", {
        from: contractOwnerAddress,
        libraries: {
            LibClaimsMask: libClaimsMask.address,
        },
    });
    console.log(`[${ diamondInit.address }] Deployed DiamondInit`);
};

export default func;
func.tags = ["RollupsDiamond"];
