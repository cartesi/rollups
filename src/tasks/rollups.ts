// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import fs from "fs";
import {
    HardhatRuntimeEnvironment,
    Network,
    TaskArguments,
} from "hardhat/types";
import { task, types } from "hardhat/config";
import { ContractExport, Export } from "hardhat-deploy/dist/types";

const exportDeployment = async (
    network: Network,
    contracts: { [name: string]: ContractExport },
    filename: string
) => {
    const exp: Export = {
        name: network.name,
        chainId: network.config.chainId?.toString() || "", // why can it be undefined?
        contracts,
    };
    fs.writeFileSync(filename, JSON.stringify(exp));
};

task("rollups:create", "Create a set of Rollups contracts")
    .addParam(
        "export",
        "File to export the deployed contracts information",
        undefined,
        types.string,
        true
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers, network } = hre;
        const MINUTE = 60; // seconds in a minute
        const HOUR = 60 * MINUTE; // seconds in an hour
        const DAY = 24 * HOUR; // seconds in a day

        const INPUT_DURATION = 1 * DAY;
        const CHALLENGE_PERIOD = 7 * DAY;
        const INPUT_LOG2_SIZE = 25;

        let signers = await ethers.getSigners();

        // Bitmask
        const Bitmask = await deployments.get("Bitmask");

        // Merkle
        const Merkle = await deployments.get("Merkle");

        // RollupsImpl
        const RollupsImpl = await deployments.deploy("RollupsImpl", {
            from: await signers[0].getAddress(),
            libraries: {
                Bitmask: Bitmask.address,
                Merkle: Merkle.address,
            },
            args: [
                INPUT_DURATION,
                CHALLENGE_PERIOD,
                INPUT_LOG2_SIZE,
                [await signers[0].getAddress()],
            ],
        });

        // we have to `require`, not `import`, because it's built by typechain
        const { RollupsImpl__factory } =
            await require("../../dist/src/types/factories/RollupsImpl__factory");

        let rollupsImpl = RollupsImpl__factory.connect(
            RollupsImpl.address,
            signers[0]
        );

        let inputAddress = await rollupsImpl.getInputAddress();
        let outputAddress = await rollupsImpl.getOutputAddress();

        let Erc20PortalImpl = await deployments.deploy("ERC20PortalImpl", {
            from: await signers[0].getAddress(),
            args: [inputAddress, outputAddress],
        });

        let EtherPortalImpl = await deployments.deploy("EtherPortalImpl", {
            from: await signers[0].getAddress(),
            args: [inputAddress, outputAddress],
        });

        console.log("Rollups Impl address: " + rollupsImpl.address);
        console.log(
            "Rollups Impl getCurrentEpoch: " +
                (await rollupsImpl.getCurrentEpoch())
        );
        console.log(
            "Rollups accumulation start: " +
                (await rollupsImpl.getInputAccumulationStart())
        );
        console.log("Input address " + inputAddress);
        console.log("Output address " + outputAddress);
        console.log("Ether Portal address " + EtherPortalImpl.address);
        console.log("ERC20 Portal address " + Erc20PortalImpl.address);

        // write export deployment file
        if (args.export) {
            network.name;
            await exportDeployment(
                network,
                {
                    RollupsImpl: {
                        address: RollupsImpl.address,
                        abi: RollupsImpl.abi,
                    },
                    EtherPortalImpl: {
                        address: EtherPortalImpl.address,
                        abi: EtherPortalImpl.abi,
                    },
                    Erc20PortalImpl: {
                        address: Erc20PortalImpl.address,
                        abi: Erc20PortalImpl.abi,
                    },
                },
                args.export
            );
        }
    });

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
