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
import { Authority__factory } from "../src/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, ethers, getNamedAccounts, network } = hre;
    const { deployer } = await getNamedAccounts();
    const [deployerSigner] = await ethers.getSigners();

    // IoTeX does not support the deterministic deployment through the contract used by hardhat-deploy
    const deterministicDeployment = network.name !== "iotex_testnet";

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
        deployerSigner
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
