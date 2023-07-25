// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
    transactional: boolean = false,
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
