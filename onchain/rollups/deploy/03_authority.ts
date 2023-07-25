// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction, DeployOptions } from "hardhat-deploy/types";
import { Authority__factory } from "../src/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, ethers, getNamedAccounts, network } = hre;
    const { deployer } = await getNamedAccounts();
    const [deployerSigner] = await ethers.getSigners();

    // IoTeX doesn't have support yet, see https://github.com/safe-global/safe-singleton-factory/issues/199
    // Chiado is not working, see https://github.com/safe-global/safe-singleton-factory/issues/201
    const nonDeterministicNetworks = ["iotex_testnet", "chiado"];
    const deterministicDeployment = !nonDeterministicNetworks.includes(
        network.name,
    );

    const opts: DeployOptions = {
        deterministicDeployment,
        from: deployer,
        log: true,
    };

    const Authority = await deployments.deploy("Authority", {
        ...opts,
        args: [deployer],
    });

    const History = await deployments.deploy("History", {
        ...opts,
        args: [Authority.address],
    });

    const authority = Authority__factory.connect(
        Authority.address,
        deployerSigner,
    );

    const currentHistory = await authority.getHistory();

    if (currentHistory != History.address) {
        const tx = await authority.setHistory(History.address);
        const receipt = await tx.wait();
        if (receipt.status == 0) {
            throw Error(`Could not link Authority to history: ${tx.hash}`);
        }
    }
};

export default func;
func.tags = ["Authority"];
