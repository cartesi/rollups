// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { ActionType, HardhatRuntimeEnvironment } from "hardhat/types";
import {
    OutputFacet__factory,
    RollupsFacet__factory,
    InputFacet__factory,
    RollupsFacet,
    InputFacet,
    OutputFacet,
} from "@cartesi/rollups";
import { RollupsArgs } from "./args";

type RollupsFacets = {
    rollupsFacet: RollupsFacet;
    inputFacet: InputFacet;
    outputFacet: OutputFacet;
};

/**
 * Connects to a Rollups diamond contract and its Rollups, Input and Output facets.
 * @param args arguments with information about which rollups to connect
 * @param hre Hardhat Runtime Environment
 * @returns three connected contracts, Rollups, Input and Output
 */
export const connect = async (
    args: RollupsArgs,
    hre: HardhatRuntimeEnvironment
): Promise<RollupsFacets> => {
    const { ethers } = hre;

    // choose a signer based on MNEMONIC and account index
    const signers = await ethers.getSigners();
    const index = args.accountIndex || 0;
    if (index < 0 || index >= signers.length) {
        throw new Error(
            `Invalid signer account index ${index}: must be between 0 and ${signers.length}`
        );
    }
    const signer = signers[index];

    // connect to RollupsFacet
    const rollupsFacet = RollupsFacet__factory.connect(args.rollups, signer);

    // connect to InputFacet
    const inputFacet = InputFacet__factory.connect(args.rollups, signer);

    // connect to OutputFacet
    const outputFacet = OutputFacet__factory.connect(args.rollups, signer);

    return {
        rollupsFacet,
        inputFacet,
        outputFacet,
    };
};

type RollupsAction<TArgs extends RollupsArgs> = (
    args: TArgs,
    rollups: RollupsFacets,
    hre: HardhatRuntimeEnvironment
) => Promise<any>;

/**
 * This is a wrapper around a hardhat task action that connects to Rollups
 * related contracts and calls a specialized action that deals with the
 * contracts.
 * @param action action that receives connected contracts
 * @returns harhat task action
 */
export function connected<TArgs extends RollupsArgs>(
    action: RollupsAction<TArgs>
): ActionType<TArgs> {
    return async (args, hre) => {
        // connect to rollups contracts
        const contracts = await connect(args, hre);

        // call the action
        return action(args, contracts, hre);
    };
}
