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

task("rollups:claim", "Send a claim to the current epoch")
    .addParam("claim", "Validator's bytes32 claim for current claimable epoch")
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        const [signer] = await ethers.getSigners();
        let claim = args.claim;

        let rollupsDeployed = await deployments.get("RollupsImpl");

        let rollups = await ethers.getContractAt(
            rollupsDeployed.abi,
            rollupsDeployed.address
        );

        const tx = await rollups.claim(claim);
        console.log(`${signer.address}: ${tx} : ${claim}`);
    });

task(
    "rollups:finalizeEpoch",
    "Finalizes epoch, if challenge period has passed",
    async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        const [signer] = await ethers.getSigners();

        let rollupsDeployed = await deployments.get("RollupsImpl");
        let rollups = await ethers.getContractAt(
            rollupsDeployed.abi,
            rollupsDeployed.address
        );

        const tx = await rollups.finalizeEpoch();
        console.log(`${signer.address}: ${tx}`);
    }
);

task(
    "rollups:getState",
    "Prints current epoch, current phase, input duration etc",
    async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        const [signer] = await ethers.getSigners();

        enum Phases {
            InputAccumulation,
            AwaitingConsensus,
            AwaitingDispute,
        }
        let rollupsDeployed = await deployments.get("RollupsImpl");
        let rollups = await ethers.getContractAt(
            rollupsDeployed.abi,
            rollupsDeployed.address
        );

        const inputDuration = await rollups.inputDuration();
        const challengePeriod = await rollups.challengePeriod();
        const currentEpoch = await rollups.getCurrentEpoch();
        const accumulationStart = await rollups.inputAccumulationStart();
        const sealingEpochTimestamp = await rollups.sealingEpochTimestamp();

        const currentPhase = await rollups.currentPhase();

        console.log(`
            current timestamp: ${
                (await ethers.provider.getBlock("latest")).timestamp
            }.
            input duration: ${inputDuration},
            challenge period: ${challengePeriod},
            current epoch: ${currentEpoch},
            accumulation start: ${accumulationStart},
            sealing epoch timestamp: ${sealingEpochTimestamp},
            current phase: ${Phases[currentPhase]},
            `);
    }
);
