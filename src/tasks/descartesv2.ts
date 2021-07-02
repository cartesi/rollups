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

task("dv2:claim", "Send a claim to the current epoch")
    .addParam(
        "claim",
        "Validator's bytes32 claim for current claimable epoch"
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        const [signer] = await ethers.getSigners();
        let claim = args.claim;
        let dv2Deployed = (await deployments.fixture())["DescartesV2Impl"];
        let dv2 = await ethers.getContractAt(dv2Deployed.abi, dv2Deployed.address);

        const tx = await dv2.claim(claim);
        console.log(`${signer.address}: ${tx} : ${claim}`);
    });
