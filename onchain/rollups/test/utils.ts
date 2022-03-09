// Copyright (C) 2022 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import client from "./client";
import { GetStateRequest } from "../generated-src/proto/stateserver_pb";
import { keccak256, defaultAbiCoder } from "ethers/lib/utils";
import { deployments, network } from "hardhat";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { BigNumber } from "ethers";
import { DeployOptions } from "hardhat-deploy/types";

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

export const getState = async (initialState: string) => {
    const request = new GetStateRequest();
    request.setJsonInitialState(initialState);

    return new Promise<string>((resolve, reject) => {
        client.getState(request, (err, response) => {
            if (err) {
                return reject(err);
            }
            return resolve(response.getJsonState());
        });
    });
};

export interface DiamondOptions {
    templateHash?: string; // defaults to 0x00
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

enum FacetCutAction {
    Add = 0,
    Replace = 1,
    Remove = 2,
}

interface FacetCut {
    facetAddress: string;
    action: FacetCutAction;
    functionSelectors: string[];
}

export const MINUTE = 60; // seconds in a minute
export const HOUR = 60 * MINUTE; // seconds in an hour
export const DAY = 24 * HOUR; // seconds in a day

export const deployDiamond = deployments.createFixture(
    async (hre: HardhatRuntimeEnvironment, options: DiamondOptions = {}) => {
        const { deployments, ethers, getNamedAccounts } = hre;
        const signers = await ethers.getSigners();
        const { deployer } = await getNamedAccounts();

        // ensure facets are deployed
        await deployments.fixture();

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

        console.log("===> Deploying CartesiDApp");

        // deploy raw diamond with diamond cut facet
        const DiamondCutFacet = await deployments.get("DiamondCutFacet");
        const diamond = await deployments.deploy("CartesiDApp", {
            ...opts,
            args: [deployer, DiamondCutFacet.address],
        });
        console.log(`[${diamond.address}] Deployed CartesiDApp`);

        // list all facets to add in a diamond cut
        const facetNames: string[] = [
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
            "ValidatorManagerFacet",
        ];

        // add the debug facet to the diamond if `debug` is true
        if (options.debug) {
            facetNames.push("DebugFacet");
        }

        console.log("===> Listing diamond facets");

        // list all facet cuts to be made
        const facetCuts: FacetCut[] = [];

        for (const facetName of facetNames) {
            const facetDeployment = await deployments.get(facetName);
            const facet = await ethers.getContractAt(
                facetName,
                facetDeployment.address
            );
            const signatures = Object.keys(facet.interface.functions);
            const selectors = signatures.reduce(
                (acc: string[], val: string) => {
                    if (val !== "init(bytes") {
                        acc.push(facet.interface.getSighash(val));
                    }
                    return acc;
                },
                []
            );
            facetCuts.push({
                facetAddress: facet.address,
                action: FacetCutAction.Add,
                functionSelectors: selectors,
            });
            console.log(`[${facet.address}] Adding ${facetName}`);
        }

        console.log("===> Executing diamond cut");

        // Default option values
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
            const { Bank } = await deployments.all();
            feeManagerBank = Bank.address;
        }

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

        // make diamond cut
        const diamondCutFacet = await ethers.getContractAt(
            "IDiamondCut",
            diamond.address
        );
        const diamondInitDeployment = await deployments.get("DiamondInit");
        const diamondInit = await ethers.getContractAt(
            "DiamondInit",
            diamondInitDeployment.address
        );
        const functionCall = diamondInit.interface.encodeFunctionData("init", [
            templateHash,
            inputDuration,
            challengePeriod,
            inputLog2Size,
            feePerClaim,
            feeManagerBank,
            feeManagerOwner,
            validators,
        ]);
        const tx = await diamondCutFacet.diamondCut(
            facetCuts,
            diamondInit.address,
            functionCall
        );
        const receipt = await tx.wait();
        if (!receipt.status) {
            throw Error(`Diamond cut failed: ${tx.hash}`);
        }

        console.log(`Diamond cut succeeded!`);

        return diamond;
    }
);

export const increaseTimeAndMine = async (duration: number) => {
    await network.provider.send("evm_increaseTime", [duration]);
    await network.provider.send("evm_mine");
};
