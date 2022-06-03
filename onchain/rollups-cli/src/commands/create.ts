// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import fs from "fs";
import { ICartesiDAppFactory } from "@cartesi/rollups";
import { ApplicationCreatedEvent } from "@cartesi/rollups/dist/src/types/contracts/ICartesiDAppFactory";
import { Wallet } from "ethers";
import { Argv } from "yargs";
import {
    BlockchainArgs,
    blockchainBuilder,
    HARDHAT_DEFAULT_MNEMONIC,
} from "../args";
import { factory } from "../connect";

interface Args extends BlockchainArgs {
    diamondOwner: string;
    templateHash?: string;
    templateHashFile?: string;
    inputDuration: number;
    challengePeriod: number;
    inputLog2Size: number;
    feePerClaim: string;
    feeManagerOwner: string;
    validators: string;
    outputFile?: string;
}

export const command = "create";
export const describe = "Instantiate rollups application";

/**
 * Process a CSV list of addresses which can also be integers representing account index from mnemonic
 * @param str CSV list of addresses or account indexes
 * @param mnemonic mnemonic to use if account index is used
 * @returns list of addresses
 */
const validators = (str: string, mnemonic: string): string[] => {
    const isIndex = (str: string): boolean =>
        str.match(/^[0-9]+$/) ? true : false;

    const mnemonicAddress = (mnemonic: string, index: number): string =>
        Wallet.fromMnemonic(mnemonic, `m/44'/60'/0'/0/${index}`).address;

    return str
        .split(",")
        .map((address) =>
            isIndex(address)
                ? mnemonicAddress(mnemonic, parseInt(address))
                : address
        );
};

const readTemplateHash = (filename: string): string => {
    if (!fs.existsSync(filename)) {
        throw new Error(`template hash file not found: ${filename}`);
    }
    return "0x" + fs.readFileSync(filename).toString("hex");
};

export const builder = (yargs: Argv<Args>) => {
    return blockchainBuilder(yargs, true)
        .option("diamondOwner", {
            describe: "Rollups contract owner",
            type: "string",
        })
        .option("templateHash", {
            describe: "Cartesi Machine template hash",
            type: "string",
        })
        .option("templateHashFile", {
            describe: "Cartesi Machine template hash file",
            type: "string",
        })
        .option("inputDuration", {
            describe: "Time window of input collection, in seconds",
            type: "number",
            default: 86400,
        })
        .option("challengePeriod", {
            describe: "Time window of challenge, in seconds",
            type: "number",
            default: 604800,
        })
        .option("inputLog2Size", {
            describe: "Log2 size of input",
            type: "number",
            default: 25,
        })
        .option("feePerClaim", {
            describe: "Fee to reward validators for claims",
            type: "string",
            default: "10000000000000000000",
        })
        .option("feeManagerOwner", {
            describe:
                "Fee Manager owner, defaults to the address of the deployer",
            type: "string",
        })
        .option("validators", {
            describe:
                "Comma separated list of validator nodes addresses. If item is a number consider as an account index of the defined MNEMONIC",
            type: "string",
            default: "0",
        })
        .option("outputFile", {
            describe: "Output file to write application address",
            type: "string",
        })
        .config();
};

export const middleware = (args: Args) => {
    console.log("middleware");
    if (args.network == "localhost" && !args.mnemonic) {
        args.mnemonic = HARDHAT_DEFAULT_MNEMONIC;
    }
};

export const handler = async (args: Args) => {
    const { deploymentFile, mnemonic, accountIndex, network, outputFile } =
        args;

    // connect to provider, use deployment address based on returned chain id of provider
    console.log(`connecting to network ${network}`);

    // connect to factory
    const factoryContract = factory(
        network,
        mnemonic,
        accountIndex,
        deploymentFile
    );

    const address = await factoryContract.signer.getAddress();
    console.log(`using account "${address}"`);

    if (!args.templateHash && !args.templateHashFile) {
        throw new Error(
            "either --templateHash or --templateHashFile must be defined"
        );
    }
    const templateHash =
        args.templateHash || readTemplateHash(args.templateHashFile!);

    // send transaction
    const config: ICartesiDAppFactory.AppConfigStruct = {
        diamondOwner: args.diamondOwner || address,
        templateHash: templateHash,
        inputDuration: args.inputDuration,
        challengePeriod: args.challengePeriod,
        inputLog2Size: args.inputLog2Size,
        feePerClaim: args.feePerClaim,
        feeManagerOwner: args.feeManagerOwner || address,
        validators: validators(args.validators, args.mnemonic!),
    };

    // print configuration
    console.log(config);

    const tx = await factoryContract.newApplication(config);
    console.log(`transaction: ${tx.hash}`);
    console.log("waiting for confirmation...");
    const receipt = await tx.wait(1);

    // find new application event in receipt
    const event = receipt.events?.find(
        (e) => e.event === "ApplicationCreated"
    ) as ApplicationCreatedEvent | undefined;
    const application = event?.args.application;

    if (application) {
        console.log(`application: ${application}`);
        if (outputFile) {
            fs.writeFileSync(outputFile, application);
        }
    }
};
