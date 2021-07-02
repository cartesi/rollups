// Copyright 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { HardhatRuntimeEnvironment, TaskArguments } from "hardhat/types";
import { task, types } from "hardhat/config";
import { BigNumber } from "ethers";
import { formatUnits } from "@ethersproject/units";

task("input:addInput", "Send an input to rollups")
    .addParam("input", "bytes to processed by the offchain machine")
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        const [signer] = await ethers.getSigners();
        let input = args.input;
        let dv2Deployed = await deployments.get("DescartesV2Impl");

        let dv2 = await ethers.getContractAt(
            dv2Deployed.abi,
            dv2Deployed.address
        );

        let inputArtifact = await deployments.getArtifact("InputImpl");

        let inputContract = await ethers.getContractAt(
            inputArtifact.abi,
            await dv2.getInputAddress()
        );

        const tx = await inputContract.addInput(input);
        console.log(`${signer.address}: ${tx} : ${input}`);
    });
