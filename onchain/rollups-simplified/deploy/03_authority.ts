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
import { History__factory } from "../src/types";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, ethers, getNamedAccounts } = hre;
    const { deployer } = await getNamedAccounts();
    const [ deployerSigner ] = await ethers.getSigners();

    const opts: DeployOptions = {
        deterministicDeployment: true,
        from: deployer,
        log: true,
    };

    const History = await deployments.deploy("History", {
        ...opts,
        args: [deployer],
    });

    const { InputBox } = await deployments.all();

    const Authority = await deployments.deploy("Authority", {
        ...opts,
        args: [deployer, InputBox.address, History.address],
    });

    const history = History__factory.connect(History.address, deployerSigner);
    const historyOwner = await history.owner();

    if (historyOwner != Authority.address) {
        const tx = await history.transferOwnership(Authority.address);
        const receipt = await tx.wait();
        if (!receipt.status) {
            throw Error(`Could not transfer ownership over History to Authority: ${tx.hash}`);
        }
    }
};

export default func;
func.dependencies = ["Input"];
func.tags = ["Authority"];
