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
import { RollupsArgs, CreateArgs, ClaimArgs } from "./args";
import {
    accountIndexParam,
    claimParams,
    createParams,
    rollupsParams,
} from "./params";
import { connect, connected } from "./connect";
import {
    taskDefs,
    TASK_CLAIM,
    TASK_CREATE,
    TASK_FINALIZE_EPOCH,
    TASK_GET_STATE,
} from "./constants";

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

            // get util contracts
            const Bitmask = await deployments.get("Bitmask");
            const Merkle = await deployments.get("Merkle");

            // process list of validators from config. If item is a number consider as an account index of the defined MNEMONIC
            const signers = await ethers.getSigners();
            const validators: string[] = args.validators
                .split(",")
                .map((address) =>
                    isIndex(address)
                        ? signers[parseInt(address)].address
                        : address
                );

            // RollupsImpl
            const RollupsImpl = await deployments.deploy("RollupsImpl", {
                from: deployer,
                libraries: {
                    Bitmask: Bitmask.address,
                    Merkle: Merkle.address,
                },
                args: [
                    args.inputDuration,
                    args.challengePeriod,
                    args.inputLog2Size,
                    validators,
                ],
                log: args.log,
            });

            const { inputContract, outputContract } = await connect(
                { rollups: RollupsImpl.address },
                hre
            );

            // deploy ETH portal
            const EtherPortalImpl = await deployments.deploy(
                "EtherPortalImpl",
                {
                    from: deployer,
                    args: [inputContract.address, outputContract.address],
                    log: args.log,
                }
            );

            // deploy ERC20 portal
            const ERC20PortalImpl = await deployments.deploy(
                "ERC20PortalImpl",
                {
                    from: deployer,
                    args: [inputContract.address, outputContract.address],
                    log: args.log,
                }
            );

            const result = {
                RollupsImpl,
                EtherPortalImpl,
                ERC20PortalImpl,
            };
            return result;
        }
    )
);

rollupsParams(
    claimParams(
        task<ClaimArgs>(
            TASK_CLAIM,
            taskDefs[TASK_CLAIM].description,
            connected(async (args, { rollupsContract }) => {
                const tx = await rollupsContract.claim(args.claim);
                console.log(
                    `Claim ${args.claim} sent to rollups ${rollupsContract.address}`
                );
                return tx;
            })
        )
    )
);

rollupsParams(
    accountIndexParam(
        task<RollupsArgs>(
            TASK_FINALIZE_EPOCH,
            taskDefs[TASK_FINALIZE_EPOCH].description,
            connected(async (_args, { rollupsContract }) => {
                const tx = await rollupsContract.finalizeEpoch();
                console.log(
                    `Finalized epoch for rollups ${rollupsContract.address}`
                );
                return tx;
            })
        )
    )
);

rollupsParams(
    task<RollupsArgs>(
        TASK_GET_STATE,
        taskDefs[TASK_GET_STATE].description,
        connected(async (_args, { rollupsContract }, hre) => {
            const { ethers } = hre;

            enum Phases {
                InputAccumulation,
                AwaitingConsensus,
                AwaitingDispute,
            }
            const storageVar = await rollupsContract.storageVar();
            const currentEpoch = await rollupsContract.getCurrentEpoch();
            const block = await ethers.provider.getBlock("latest");

            const result = {
                currentTimestamp: block.timestamp,
                inputDuration: storageVar.inputDuration,
                challengePeriod: storageVar.challengePeriod,
                currentEpoch: currentEpoch.toNumber(),
                accumulationStart: storageVar.inputAccumulationStart,
                sealingEpochTimestamp: storageVar.sealingEpochTimestamp,
                currentPhase: Phases[storageVar.currentPhase_int],
            };
            console.log(JSON.stringify(result, null, 2));
            return result;
        })
    )
);
