// Copyright Cartesi Pte. Ltd.

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
    const { deployments, getNamedAccounts, network } = hre;
    const { deployer } = await getNamedAccounts();

    // IoTeX does not support the deterministic deployment through the contract used by hardhat-deploy
    const deterministicDeployment = network.name !== "iotex_testnet";

    const opts: DeployOptions = {
        deterministicDeployment,
        from: deployer,
        log: true,
    };

    const InputBox = await deployments.deploy("InputBox", opts);

    await deployments.deploy("EtherPortal", {
        ...opts,
        args: [InputBox.address],
    });
    await deployments.deploy("ERC20Portal", {
        ...opts,
        args: [InputBox.address],
    });
    await deployments.deploy("ERC721Portal", {
        ...opts,
        args: [InputBox.address],
    });
    await deployments.deploy("ERC1155SinglePortal", {
        ...opts,
        args: [InputBox.address],
    });
    await deployments.deploy("ERC1155BatchPortal", {
        ...opts,
        args: [InputBox.address],
    });
    await deployments.deploy("DAppAddressRelay", {
        ...opts,
        args: [InputBox.address],
    });
};

export default func;
func.tags = ["Input"];
