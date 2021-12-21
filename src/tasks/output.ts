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

task("output:executeVoucher", "execute a voucher")
    .addParam(
        "destination",
        "the destination address that is called for execution"
    )
    .addParam(
        "payload",
        "the abi encoding of the called function and arguments"
    )
    .addParam(
        "proof",
        "proof for the voucher being valid. Should be wrapped as a JSON string."
    )
    .addOptionalParam(
        "signer",
        "account index of the signer adding the input",
        0,
        types.int
    )
    .setAction(async (args: TaskArguments, hre: HardhatRuntimeEnvironment) => {
        const { deployments, ethers } = hre;

        let destination = args.destination;
        let payload = args.payload;
        let proof = JSON.parse(args.proof); // string to JSON object

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

        let outputArtifact = await deployments.getArtifact("OutputImpl");
        let outputContract = await ethers.getContractAt(
            outputArtifact.abi,
            await rollups.getOutputAddress()
        );

        const tx = await outputContract.executeVoucher(
            destination,
            payload,
            proof
        );
        const events = (await tx.wait()).events;
        const voucherExecutedEvent = getEvent(
            "VoucherExecuted",
            outputContract,
            events
        );

        if (!voucherExecutedEvent) {
            console.log(
                `Failed to execute payload '${payload}' at destination '${destination}' with proof '${proof}' (signer: ${signer.address}, tx: ${tx.hash})\n`
            );
        } else {
            const voucherPosition =
                voucherExecutedEvent.args.voucherPosition.toString();
            console.log(
                `Executed voucher at position '${voucherPosition}' (signer: ${signer.address}, tx: ${tx.hash})`
            );
        }
    });
