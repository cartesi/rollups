// Copyright Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import fs from "fs";
import fse from "fs-extra";
import { Argv, CommandModule } from "yargs";
import { Overrides } from "@ethersproject/contracts";
import { parseUnits } from "@ethersproject/units";
import { ApplicationCreatedEvent } from "@cartesi/rollups/dist/src/types/contracts/dapp/ICartesiDAppFactory";

import { BlockchainArgs, blockchainBuilder } from "../args";
import { factory } from "../connect";
import { safeHandler } from "../util";

interface Args extends BlockchainArgs {
    dappOwner?: string;
    consensusAddress: string;
    templateHash?: string;
    templateHashFile?: string;
    salt?: string;
    outputFile?: string;
    gasPrice?: number;
    gasLimit?: number;
}

/**
 * Read a Cartesi Machine hash from its internal `hash` binary file
 * @param filename path of cartesi machine internal hash file
 * @returns Hash of the machine as string, prefixed by 0x
 */
const readTemplateHash = (filename: string): string => {
    if (!fs.existsSync(filename)) {
        throw new Error(`template hash file not found: ${filename}`);
    }
    return "0x" + fs.readFileSync(filename).toString("hex");
};

const builder = (yargs: Argv<{}>): Argv<Args> => {
    return blockchainBuilder(yargs, true)
        .option("dappOwner", {
            describe: "Rollups contract owner",
            type: "string",
        })
        .option("consensusAddress", {
            describe: "Consensus contract address",
            type: "string",
            demandOption: true,
        })
        .option("templateHash", {
            describe: "Cartesi Machine template hash",
            type: "string",
        })
        .option("templateHashFile", {
            describe: "Cartesi Machine template hash file",
            type: "string",
        })
        .option("salt", {
            describe: "Salt used in deterministic deployment",
            type: "string",
        })
        .option("outputFile", {
            describe:
                "Output file to write application information in JSON format",
            type: "string",
        })
        .option("gasPrice", {
            describe: "Gas price to use for deployment, in GWei",
            type: "number",
        })
        .option("gasLimit", {
            describe: "Gas limit to use for deployment",
            type: "number",
        })
        .config();
};

const handler = safeHandler<Args>(async (args) => {
    const {
        deploymentFile,
        mnemonic,
        accountIndex,
        rpc,
        outputFile,
        gasPrice,
        gasLimit,
    } = args;

    // connect to provider, use deployment address based on returned chain id of provider
    console.log(`connecting to ${rpc}`);

    // connect to factory
    const factoryContract = await factory(
        rpc,
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

    const overrides: Overrides = {};
    if (gasPrice) {
        overrides.gasPrice = parseUnits(gasPrice.toString(), "gwei");
    }
    if (gasLimit) {
        overrides.gasLimit = gasLimit;
    }

    const consensusAddress = args.consensusAddress;
    const dappOwner = args.dappOwner || address;

    let tx;
    if (args.salt) {
        tx = await factoryContract[
            "newApplication(address,address,bytes32,bytes32)"
        ](consensusAddress, dappOwner, templateHash, args.salt, overrides);
    } else {
        tx = await factoryContract["newApplication(address,address,bytes32)"](
            consensusAddress,
            dappOwner,
            templateHash,
            overrides
        );
    }

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
            console.log(`writing application address to ${outputFile}`);
            fse.outputFileSync(
                outputFile,
                JSON.stringify(
                    {
                        address: application,
                        blockHash: receipt.blockHash,
                        blockNumber: receipt.blockNumber,
                        transactionHash: receipt.transactionHash,
                    },
                    null,
                    4
                )
            );
        }
    }
});

const cmd: CommandModule<{}, Args> = {
    command: "create",
    describe: "Instantiate rollups application",
    builder,
    handler,
};

export default cmd;
