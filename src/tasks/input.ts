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
import { getEvent } from "./eventUtil";

task("input:addInput", "Send an input to rollups")
    .addParam("input", "bytes to processed by the offchain machine")
    .addOptionalParam(
        "signer",
        "account index of the signer adding the input",
        0,
        types.int
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        let input = args.input;
        let signerIndex = args.signer;
        const signers = await ethers.getSigners();
        if (signerIndex < 0 || signerIndex >= signers.length) {
            console.error(
                `Invalid signer account index ${signerIndex}: must be between 0 and ${signers.length}`
            );
            return;
        }
        const signer = signers[signerIndex];

        let rollupsDeployed = await deployments.get("RollupsImpl");

        let rollups = await ethers.getContractAt(
            rollupsDeployed.abi,
            rollupsDeployed.address
        );

        let inputArtifact = await deployments.getArtifact("InputImpl");

        let inputContract = await ethers.getContractAt(
            inputArtifact.abi,
            await rollups.getInputAddress()
        );

        const tx = await inputContract.addInput(input);

        const events = (await tx.wait()).events;
        const inputAddedEvent = getEvent("InputAdded", inputContract, events);
        if (!inputAddedEvent) {
            console.log(
                `Failed to add input '${input}' (signer: ${signer.address}, tx: ${tx.hash})\n`
            );
        } else {
            const epochNumber = inputAddedEvent.args._epochNumber.toString();
            const timestamp = inputAddedEvent.args._timestamp.toString();
            console.log(
                `Added input '${input}' to epoch '${epochNumber}' (timestamp: ${timestamp}, signer: ${signer.address}, tx: ${tx.hash})`
            );
        }
    });

task("input:addRepeatedInputs", "Send an input to rollups")
    .addParam("input", "bytes to processed by the offchain machine")
    .addParam("count", "number of repetitions")
    .addOptionalParam(
        "signer",
        "account index of the signer adding the input",
        0,
        types.int
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;
        let input = args.input;
        let repetitions = args.count;
        let signerIndex = args.signer;
        const signers = await ethers.getSigners();
        if (signerIndex < 0 || signerIndex >= signers.length) {
            console.error(
                `Invalid signer account index ${signerIndex}: must be between 0 and ${signers.length}`
            );
            return;
        }
        const signer = signers[signerIndex];

        let rollupsDeployed = await deployments.get("RollupsImpl");

        let rollups = await ethers.getContractAt(
            rollupsDeployed.abi,
            rollupsDeployed.address
        );

        let inputArtifact = await deployments.getArtifact("InputImpl");

        let inputContract = await ethers.getContractAt(
            inputArtifact.abi,
            await rollups.getInputAddress()
        );

        for (let i = 0; i < repetitions; i++) {
            const tx = await inputContract.addInput(input);
            const events = (await tx.wait()).events;
            const inputAddedEvent = getEvent(
                "InputAdded",
                inputContract,
                events
            );
            if (!inputAddedEvent) {
                console.log(
                    `Failed to add input '${input}' (signer: ${signer.address}, tx: ${tx.hash})\n`
                );
            } else {
                const epochNumber =
                    inputAddedEvent.args._epochNumber.toString();
                const timestamp = inputAddedEvent.args._timestamp.toString();
                console.log(
                    `Added input '${input}' to epoch '${epochNumber}' (timestamp: ${timestamp}, signer: ${signer.address}, tx: ${tx.hash})`
                );
            }
        }
    });
