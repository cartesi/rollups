// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { keccak256, defaultAbiCoder } from "ethers/lib/utils";
import { deployments, network } from "hardhat";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { BytesLike } from "@ethersproject/bytes";
import { BigNumber } from "ethers";
import { DeployOptions } from "hardhat-deploy/types";
import { ServiceError } from "@grpc/grpc-js";
import createClient from "./client";
import { GetStateResponse__Output } from "../generated-src/proto/StateServer/GetStateResponse";
import { getFacetCuts, productionFacetNames } from "../src/utils";
import {
    CartesiDAppFactory,
    ICartesiDAppFactory,
    CartesiDAppFactory__factory,
    CartesiDApp__factory,
    SimpleNFT,
    SimpleNFT__factory,
} from "../src/types";

const client = createClient();

// Calculate input hash based on
// input: data itself interpreted by L2
// blockNumber: `block.number'
// blockTimestamp: `block.timestamp'
// epochIndex: epoch index
// inputIndex: input index
export const getInputHash = (
    input: any,
    sender: string,
    blockNumber: number,
    blockTimestamp: number,
    epochIndex: number,
    inputIndex: number
) => {
    // combine input attributes into one
    const metadata = defaultAbiCoder.encode(
        ["uint", "uint", "uint", "uint", "uint"],
        [sender, blockNumber, blockTimestamp, epochIndex, inputIndex]
    );

    // keccak the metadata and the input
    const keccak_metadata = keccak256(metadata);
    const keccak_input = keccak256(input);

    // combine the two keccaks into one
    const abi_metadata_input = defaultAbiCoder.encode(
        ["uint", "uint"],
        [keccak_metadata, keccak_input]
    );

    // keccak the combined keccaks
    const input_hash = keccak256(abi_metadata_input);

    // return the input hash
    return input_hash;
};

export const getState = async (jsonInitialState: string) => {
    return new Promise<string>((resolve, reject) => {
        client.GetState(
            { jsonInitialState },
            (
                err: ServiceError | null,
                response: GetStateResponse__Output | undefined
            ) => {
                if (err || !response?.jsonState) {
                    return reject(err ?? `no response`);
                }
                return resolve(response.jsonState);
            }
        );
    });
};

export interface TestBankOptions {
    initialSupply?: number | BigNumber; // defaults to 1000000
}

export const deployTestBank = deployments.createFixture(
    async (hre: HardhatRuntimeEnvironment, options: TestBankOptions = {}) => {
        const { deployments, getNamedAccounts } = hre;
        const { deployer } = await getNamedAccounts();

        let initialSupply = options.initialSupply || 1000000;

        // deploy token
        const SimpleToken = await deployments.deploy("SimpleToken", {
            from: deployer,
            args: [initialSupply],
        });

        // deploy bank
        const Bank = await deployments.deploy("Bank", {
            from: deployer,
            args: [SimpleToken.address],
        });

        return {
            Bank,
            SimpleToken,
        };
    }
);

export interface FactoryOptions {
    feeManagerBank?: string; // defaults to Bank that uses CTSI
    simpleFeeManagerBank?: boolean; // if true, deploys Bank with SimpleToken
    debug?: boolean; // defaults to false
}

export const deployFactory = deployments.createFixture(
    async (hre: HardhatRuntimeEnvironment, options: FactoryOptions = {}) => {
        const { deployments, ethers, getNamedAccounts } = hre;
        const signers = await ethers.getSigners();
        const { deployer } = await getNamedAccounts();

        const opts: DeployOptions = {
            from: deployer,
            log: true,
        };

        // deploy the debug facet if `debug` is true
        if (options.debug) {
            const LibClaimsMask = await deployments.get("LibClaimsMask");
            await deployments.deploy("DebugFacet", {
                ...opts,
                libraries: {
                    LibClaimsMask: LibClaimsMask.address,
                },
            });
        }

        // list all facet names
        let facetNames: string[];
        if (options.debug) {
            facetNames = [...productionFacetNames, "DebugFacet"];
        } else {
            facetNames = [...productionFacetNames];
        }

        // list all facet cuts
        const facetCuts = await getFacetCuts(hre, facetNames);

        const { DiamondCutFacet, DiamondInit, Bank } = await deployments.all();

        let feeManagerBank;
        if (options.feeManagerBank) {
            feeManagerBank = options.feeManagerBank;
        } else if (options.simpleFeeManagerBank) {
            let tokenSupply = 1000000;
            const tokenDeployment = await deployments.deploy("SimpleToken", {
                ...opts,
                args: [tokenSupply],
            });
            const bankDeployment = await deployments.deploy("Bank", {
                ...opts,
                args: [tokenDeployment.address],
            });
            feeManagerBank = bankDeployment.address;
        } else {
            feeManagerBank = Bank.address;
        }

        let factoryConfig: CartesiDAppFactory.FactoryConfigStruct = {
            diamondCutFacet: DiamondCutFacet.address,
            diamondInit: DiamondInit.address,
            feeManagerBank: feeManagerBank,
            diamondCut: facetCuts,
        };

        const factoryDeployment = await deployments.deploy(
            "CartesiDAppFactory",
            {
                ...opts,
                args: [factoryConfig],
            }
        );

        console.log(
            `[${factoryDeployment.address}] CartesiDAppFactory deployed`
        );

        const factory = CartesiDAppFactory__factory.connect(
            factoryDeployment.address,
            signers[0]
        );

        return factory;
    }
);

export interface DiamondOptions {
    diamondOwner?: string; // defaults to deployer
    templateHash?: BytesLike; // defaults to 0x00
    inputDuration?: number | BigNumber; // defaults to 1 day
    challengePeriod?: number | BigNumber; // defaults to 7 days
    inputLog2Size?: number | BigNumber; // defaults to 8 (thus, 2^8)
    feePerClaim?: number | BigNumber; // defaults to 10 tokens
    feeManagerBank?: string; // defaults to Bank that uses CTSI
    simpleFeeManagerBank?: boolean; // if true, deploys Bank with SimpleToken
    feeManagerOwner?: string; // defaults to the first signer
    validators?: string[]; // defaults to the 8 first signers
    debug?: boolean; // defaults to false
}

export const MINUTE = 60; // seconds in a minute
export const HOUR = 60 * MINUTE; // seconds in an hour
export const DAY = 24 * HOUR; // seconds in a day

export const deployDiamond = deployments.createFixture(
    async (hre: HardhatRuntimeEnvironment, options: DiamondOptions = {}) => {
        const { deployments, ethers, getNamedAccounts } = hre;
        const signers = await ethers.getSigners();
        const { deployer } = await getNamedAccounts();

        const opts: DeployOptions = {
            from: deployer,
            log: true,
        };

        // Default option values
        let diamondOwner = options.diamondOwner
            ? options.diamondOwner
            : deployer;
        let templateHash = options.templateHash
            ? options.templateHash
            : "0x0000000000000000000000000000000000000000000000000000000000000000";
        let inputDuration = options.inputDuration
            ? options.inputDuration
            : 1 * DAY;
        let challengePeriod = options.challengePeriod
            ? options.challengePeriod
            : 7 * DAY;
        let inputLog2Size = options.inputLog2Size ? options.inputLog2Size : 8;
        let feePerClaim = options.feePerClaim ? options.feePerClaim : 10;
        let feeManagerOwner = options.feeManagerOwner
            ? options.feeManagerOwner
            : deployer;

        let validators: string[] = [];
        if (options.validators) {
            validators = options.validators;
        } else {
            // add up to 8 signers to `validators`
            for (const signer of signers) {
                const signerAddress = await signer.getAddress();
                validators.push(signerAddress);
                if (validators.length == 8) break;
            }
        }

        // ensure factory is deployed
        const factory = await deployFactory({
            debug: options.debug,
            feeManagerBank: options.feeManagerBank,
            simpleFeeManagerBank: options.simpleFeeManagerBank,
        });

        let appConfig: ICartesiDAppFactory.AppConfigStruct = {
            diamondOwner,
            templateHash,
            inputDuration,
            challengePeriod,
            inputLog2Size,
            feePerClaim,
            feeManagerOwner,
            validators,
        };

        const tx = await factory.newApplication(appConfig);
        const receipt = await tx.wait();
        if (!receipt.status) {
            throw Error(`Application creation failed: ${tx.hash}`);
        }

        let eventFilter = factory.filters.ApplicationCreated();
        let events = await factory.queryFilter(
            eventFilter,
            receipt.blockNumber
        );
        const { application } = events[events.length - 1].args;
        console.log(`[${application}] CartesiDApp deployed`);
        return CartesiDApp__factory.connect(application, signers[0]);
    }
);

export interface SimpleNFTOptions {
    tokenIds?: number[];
}

export const deploySimpleNFT = deployments.createFixture(
    async (hre: HardhatRuntimeEnvironment, options: SimpleNFTOptions = {}) => {
        const { deployments, ethers, getNamedAccounts } = hre;
        const signers = await ethers.getSigners();
        const { deployer } = await getNamedAccounts();

        const opts: DeployOptions = {
            from: deployer,
            log: true,
        };

        let tokenIds: number[] = options.tokenIds || [];
        const simpleNFTDeployment = await deployments.deploy("SimpleNFT", {
            ...opts,
            args: [tokenIds],
        });

        const simpleNFT = SimpleNFT__factory.connect(
            simpleNFTDeployment.address,
            signers[0]
        );

        return simpleNFT;
    }
);

export const increaseTimeAndMine = async (duration: number) => {
    await network.provider.send("evm_increaseTime", [duration]);
    await network.provider.send("evm_mine");
};
