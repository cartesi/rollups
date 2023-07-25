// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
