// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { ethers } from "ethers";
import { Argv } from "yargs";

export const HARDHAT_DEFAULT_MNEMONIC =
    "test test test test test test test test test test test junk";

export interface BlockchainArgs {
    rpc: string;
    mnemonic?: string;
    accountIndex: number;
    deploymentFile?: string;
}

export const blockchainBuilder = (
    yargs: Argv<{}>,
    transactional: boolean = false
): Argv<BlockchainArgs> => {
    return yargs
        .option("rpc", {
            describe: "JSON-RPC URL",
            type: "string",
            demandOption: true,
        })
        .option("mnemonic", {
            describe: "Wallet mnemonic",
            type: "string",
            demandOption: transactional, // required if need to send transactions
        })
        .option("accountIndex", {
            describe: "Account index from mnemonic",
            type: "number",
            default: 0,
        })
        .option("deploymentFile", {
            describe: "Contracts deployment file",
            type: "string",
        });
};

/**
 * Validator for mnemonic value
 * @param value mnemonic words separated by space
 * @returns true if valid, false if invalid
 */
export const mnemonicValidator = (value: string) => {
    try {
        ethers.Wallet.fromMnemonic(value);
        return true;
    } catch (e) {
        return "Invalid mnemonic";
    }
};
