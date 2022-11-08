// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction, DeployOptions } from "hardhat-deploy/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, getNamedAccounts } = hre;
    const { deployer } = await getNamedAccounts();

    const opts: DeployOptions = {
        deterministicDeployment: true,
        from: deployer,
        log: true,
    };

    const Bitmask = await deployments.get("Bitmask");
    const MerkleV2 = await deployments.get("MerkleV2");

    await deployments.deploy("ERC20PortalFacet", opts);
    await deployments.deploy("ERC721PortalFacet", opts);
    await deployments.deploy("ERC1155PortalFacet", opts);
    await deployments.deploy("EtherPortalFacet", opts);
    await deployments.deploy("FeeManagerFacet", opts);
    await deployments.deploy("InputFacet", opts);
    await deployments.deploy("OutputFacet", {
        ...opts,
        libraries: {
            Bitmask: Bitmask.address,
            MerkleV2: MerkleV2.address,
        },
    });
    await deployments.deploy("RollupsFacet", opts);
    await deployments.deploy("ValidatorManagerFacet", opts);
};

export default func;
func.tags = ["RollupsFacets"];
