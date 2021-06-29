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

// Note: This component currently has dependencies that are licensed under the GU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { deployments, ethers } from 'hardhat'
import { expect, use } from 'chai'
import { solidity } from 'ethereum-waffle';
import { Signer } from 'ethers'
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";

//import { DescartesV2Impl }  from "../dist/src/types/DescartesV2Impl";
const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, getNamedAccounts } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();
    const { CartesiToken } = await deployments.all();

    let signers = await ethers.getSigners();
    //let descartesV2Impl: DescartesV2Impl;


    // Bitmask
    const bitMaskLibrary = await deployments.deploy(
        "Bitmask", 
        {
            from: await signers[0].getAddress()
        }
    );
    const bitMaskAddress = bitMaskLibrary.address;

    // CartesiMath
    const cartesiMathFactory = await ethers.getContractFactory(
        "CartesiMath",
        {
            signer: signers[0]
        }
    );

    let cartesiMath = await cartesiMathFactory.deploy();
    const cartesiMathAddress = cartesiMath.address;

    // Merkle
    const merkleFactory = await ethers.getContractFactory(
        "Merkle",
        {
            signer: signers[0],
            libraries: {
                CartesiMath: cartesiMathAddress 
            }
        }
    );

    let merkle = await merkleFactory.deploy();
    const merkleAddress = merkle.address;

    const descartesV2Factory = await ethers.getContractFactory(
      "DescartesV2Impl",
      {
        signer: signers[0],
        libraries: {
          Bitmask: bitMaskAddress,
          Merkle: merkleAddress
        }
      }
    )
    let descartesV2Impl = await descartesV2Factory.deploy(1,
          2,
          5,
          5,
          [await signers[0].getAddress()]
    );

    let inputAddress = await descartesV2Impl.getInputAddress();
    let outputAddress =  await descartesV2Impl.getOutputAddress();

    let portalFactory = await ethers.getContractFactory(
        "PortalImpl",
        {
            signer: signers[0],
        }
    )
    
    let portalImpl = await portalFactory.deploy(inputAddress, outputAddress)

    console.log("Descartes V2 Impl address: " + descartesV2Impl.address);
    console.log("Descartes V2 Impl getCurrentEpoch: " + await descartesV2Impl.getCurrentEpoch());
    console.log("Descartes V2 accumulation start: " + await descartesV2Impl.inputAccumulationStart());
    console.log("Input address " + inputAddress);
    console.log("Output address " + outputAddress);
    console.log("Portal address " + portalImpl.address);


};

export default func;
export const tags = ["DescartesV2Impl", "PortalImpl"];
