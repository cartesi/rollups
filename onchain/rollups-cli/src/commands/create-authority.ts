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
import { AuthorityCreatedEvent } from "@cartesi/rollups/dist/src/types/contracts/consensus/authority/IAuthorityFactory";

import { BlockchainArgs, blockchainBuilder } from "../args";
import { authorityFactory, authorityHistoryPairFactory } from "../connect";
import { safeHandler } from "../util";

interface Args extends BlockchainArgs {
    authorityOwner?: string;
    salt?: string;
    outputFile?: string;
    gasPrice?: number;
    gasLimit?: number;
}

const builder = (yargs: Argv<{}>): Argv<Args> => {
    return blockchainBuilder(yargs, true)
        .option("authorityOwner", {
            describe: "Authority contract owner",
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

    // connect to Authority-History pair factory
    const factoryContract = await authorityHistoryPairFactory(
        rpc,
        mnemonic,
        accountIndex,
        deploymentFile
    );

    const address = await factoryContract.signer.getAddress();
    console.log(`using account "${address}"`);

    const overrides: Overrides = {};
    if (gasPrice) {
        overrides.gasPrice = parseUnits(gasPrice.toString(), "gwei");
    }
    if (gasLimit) {
        overrides.gasLimit = gasLimit;
    }

    const authorityOwner = args.authorityOwner || address;

    let tx;
    if (args.salt) {
        tx = await factoryContract["newAuthorityHistoryPair(address,bytes32)"](
            authorityOwner,
            args.salt,
            overrides
        );
    } else {
        tx = await factoryContract["newAuthorityHistoryPair(address)"](
            authorityOwner,
            overrides
        );
    }

    console.log(`transaction: ${tx.hash}`);
    console.log("waiting for confirmation...");
    const receipt = await tx.wait(1);

    const authorityFactoryContract = await authorityFactory(
        rpc,
        mnemonic,
        accountIndex,
        deploymentFile
    );

    let authority: string | undefined;
    for (const log of receipt.logs) {
        if (log.address == authorityFactoryContract.address) {
            const event = authorityFactoryContract.interface.parseLog(log);
            if (event.name == "AuthorityCreated") {
                authority = event.args.authority;
                break;
            }
        }
    }

    if (authority) {
        console.log(`authority: ${authority}`);
        if (outputFile) {
            console.log(`writing authority address to ${outputFile}`);
            fse.outputFileSync(
                outputFile,
                JSON.stringify(
                    {
                        address: authority,
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
    command: "create-authority",
    describe: "Instantiate rollups authority",
    builder,
    handler,
};

export default cmd;
