// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { task } from "hardhat/config";
import "@nomiclabs/hardhat-ethers/internal/type-extensions";
import "hardhat-deploy/dist/src/type-extensions";
import { BigNumber } from "ethers";

import { RollupsArgs, CreateArgs, ClaimArgs } from "./args";
import { accountIndexParam, claimParams, createParams } from "./params";
import { connected } from "./connect";
import {
    taskDefs,
    TASK_CLAIM,
    TASK_CREATE,
    TASK_FINALIZE_EPOCH,
    TASK_GET_STATE,
} from "./constants";
import {
    CartesiDAppFactory__factory,
    ICartesiDAppFactory,
} from "@cartesi/rollups";

// return true if string is an unsigned integer
const isIndex = (str: string): boolean =>
    str.match(/^[0-9]+$/) ? true : false;

createParams(
    task<CreateArgs>(
        TASK_CREATE,
        taskDefs[TASK_CREATE].description,
        async (args, hre) => {
            const { deployments, ethers } = hre;
            const { deployer } = await hre.getNamedAccounts();

            // process list of validators from config. If item is a number consider as an account index of the defined MNEMONIC
            const signers = await ethers.getSigners();
            const diamondOwner = deployer;
            const validators: string[] = args.validators
                .split(",")
                .map((address) =>
                    isIndex(address)
                        ? signers[parseInt(address)].address
                        : address
                );

            // get pre-deployed factory artifact
            const { CartesiDAppFactory } = await deployments.all();
            const factory = CartesiDAppFactory__factory.connect(
                CartesiDAppFactory.address,
                signers[0]
            );

            // set application configurations from arguments
            let appConfig: ICartesiDAppFactory.AppConfigStruct = {
                diamondOwner: diamondOwner,
                templateHash: args.templateHash,
                inputDuration: args.inputDuration,
                challengePeriod: args.challengePeriod,
                inputLog2Size: args.inputLog2Size,
                feePerClaim: BigNumber.from(args.feePerClaim),
                feeManagerOwner: args.feeManagerOwner || deployer,
                validators: validators,
            };

            // order new application from the factory
            const tx = await factory.newApplication(appConfig);
            if (args.log) {
                process.stdout.write(
                    `deploying "${appConfig.templateHash}" (tx: ${tx.hash})...: `
                );
            }
            const receipt = await tx.wait();
            if (!receipt.status) {
                throw Error(`Application creation failed: ${tx.hash}`);
            }

            // query application address from the event log
            const eventFilter = factory.filters.ApplicationCreated();
            const events = await factory.queryFilter(
                eventFilter,
                receipt.blockNumber
            );
            for (const event of events) {
                const { application, config } = event.args;
                if (config.diamondOwner == diamondOwner) {
                    if (args.log) {
                        process.stdout.write(
                            `deployed at ${application} with ${receipt.gasUsed} gas\n`
                        );
                    }
                    return event;
                }
            }
            throw Error(`Could not find ApplicationCreated event in the logs`);
        }
    )
);

claimParams(
    task<ClaimArgs>(
        TASK_CLAIM,
        taskDefs[TASK_CLAIM].description,
        connected(async (args, { rollupsFacet }) => {
            const tx = await rollupsFacet.claim(args.claim);
            console.log(
                `Claim ${args.claim} sent to rollups ${rollupsFacet.address}`
            );
            return tx;
        })
    )
);

accountIndexParam(
    task<RollupsArgs>(
        TASK_FINALIZE_EPOCH,
        taskDefs[TASK_FINALIZE_EPOCH].description,
        connected(async (_args, { rollupsFacet }) => {
            const tx = await rollupsFacet.finalizeEpoch();
            console.log(`Finalized epoch for rollups ${rollupsFacet.address}`);
            return tx;
        })
    )
);

task<RollupsArgs>(
    TASK_GET_STATE,
    taskDefs[TASK_GET_STATE].description,
    connected(async (_args, { rollupsFacet }, hre) => {
        const { ethers } = hre;

        enum Phases {
            InputAccumulation,
            AwaitingConsensus,
            AwaitingDispute,
        }

        const templateHash = await rollupsFacet.getTemplateHash();
        const inputDuration = await rollupsFacet.getInputDuration();
        const challengePeriod = await rollupsFacet.getChallengePeriod();
        const currentEpoch = await rollupsFacet.getCurrentEpoch();
        const inputAccumulationStart =
            await rollupsFacet.getInputAccumulationStart();
        const sealingEpochTimestamp =
            await rollupsFacet.getSealingEpochTimestamp();
        const currentPhase = await rollupsFacet.getCurrentPhase();
        const block = await ethers.provider.getBlock("latest");

        const result = {
            currentTimestamp: block.timestamp,
            templateHash: templateHash,
            inputDuration: inputDuration,
            challengePeriod: challengePeriod,
            currentEpoch: currentEpoch.toNumber(),
            accumulationStart: inputAccumulationStart,
            sealingEpochTimestamp: sealingEpochTimestamp,
            currentPhase: Phases[currentPhase],
        };
        console.log(JSON.stringify(result, null, 2));
        return result;
    })
);
