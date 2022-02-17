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

import { RollupsArgs, CreateArgs, ClaimArgs } from "./args";
import {
    accountIndexParam,
    claimParams,
    createParams,
    rollupsParams,
} from "./params";
import { connected } from "./connect";
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

enum FacetCutAction {
    Add = 0,
    Replace = 1,
    Remove = 2,
}

interface FacetCut {
    facetAddress: string,
    action: FacetCutAction,
    functionSelectors: string[],
}

createParams(
    task<CreateArgs>(
        TASK_CREATE,
        taskDefs[TASK_CREATE].description,
        async (args, hre) => {
            const { deployments, ethers } = hre;
            const { deployer } = await hre.getNamedAccounts();

            // process list of validators from config. If item is a number consider as an account index of the defined MNEMONIC
            const signers = await ethers.getSigners();
            const validators: string[] = args.validators
                .split(",")
                .map((address) =>
                    isIndex(address)
                        ? signers[parseInt(address)].address
                        : address
                );

            // deploy raw diamond with only the diamond cut facet
            const diamondCutFacetDeployment = await deployments.get('DiamondCutFacet');
            const diamond = await deployments.deploy("Diamond", {
                from: deployer,
                args: [
                    deployer,
                    diamondCutFacetDeployment.address,
                ],
                log: args.log,
            });

            // list all facets to add in a diamond cut
            const facetNames : string[] = [
                // essential facets
                "DiamondLoupeFacet",
                "OwnershipFacet",
                // rollups-related facets
                "ERC20PortalFacet",
                "ERC721PortalFacet",
                "EtherPortalFacet",
                "FeeManagerFacet",
                "InputFacet",
                "OutputFacet",
                "RollupsFacet",
                "SERC20PortalFacet",
                "ValidatorManagerFacet",
            ];

            // list all facet cuts to be made
            const facetCuts : FacetCut[] = [];

            for (const facetName of facetNames) {
                const facetDeployment = await deployments.get(facetName);
                const facetArtifact = await deployments.getArtifact(facetName);
                const facet = await ethers.getContractAt(facetArtifact.abi, facetDeployment.address);
                const signatures = Object.keys(facet.interface.functions);
                const selectors = signatures.reduce((acc: string[], val: string) => {
                    if (val !== 'init(bytes') {
                        acc.push(facet.interface.getSighash(val));
                    }
                    return acc;
                }, []);
                facetCuts.push({
                    facetAddress: facet.address,
                    action: FacetCutAction.Add,
                    functionSelectors: selectors,
                });
            }

            // make diamond cut
            const diamondCutArtifact = await deployments.getArtifact('IDiamondCut');
            const diamondCutFacet = await ethers.getContractAt(diamondCutArtifact.abi, diamond.address);
            const diamondInitDeployment = await deployments.get('DiamondInit');
            const diamondInit = await ethers.getContractAt(diamondInitDeployment.abi, diamondInitDeployment.address);
            const calldata = diamondInit.interface.encodeFunctionData('init', [
                args.inputDuration,
                args.challengePeriod,
                args.inputLog2Size,
                args.feePerClaim,
                args.erc20ForFee,
                args.feeManagerOwner || deployer,
                validators,
                args.erc20ForPortal,
            ]);
            const tx = await diamondCutFacet.diamondCut(facetCuts, diamondInit.address, calldata);
            const receipt = await tx.wait();
            if (!receipt.status) {
                throw Error(`Diamond cut failed: ${tx.hash}`);
            }

            return diamond;
        }
    )
);

rollupsParams(
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
    )
);

rollupsParams(
    accountIndexParam(
        task<RollupsArgs>(
            TASK_FINALIZE_EPOCH,
            taskDefs[TASK_FINALIZE_EPOCH].description,
            connected(async (_args, { rollupsFacet }) => {
                const tx = await rollupsFacet.finalizeEpoch();
                console.log(
                    `Finalized epoch for rollups ${rollupsFacet.address}`
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
        connected(async (_args, { rollupsFacet }, hre) => {
            const { ethers } = hre;

            enum Phases {
                InputAccumulation,
                AwaitingConsensus,
                AwaitingDispute,
            }

            const inputDuration = await rollupsFacet.getInputDuration();
            const challengePeriod = await rollupsFacet.getChallengePeriod();
            const currentEpoch = await rollupsFacet.getCurrentEpoch();
            const inputAccumulationStart = await rollupsFacet.getInputAccumulationStart();
            const sealingEpochTimestamp = await rollupsFacet.getSealingEpochTimestamp();
            const currentPhase = await rollupsFacet.getCurrentPhase();
            const block = await ethers.provider.getBlock("latest");

            const result = {
                currentTimestamp: block.timestamp,
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
    )
);
