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
import { InputImpl, OutputImpl, RollupsImpl } from "../types";
import { RollupsArgs } from "./args";

/**
 * Connects to a Rollups contract and its Input and Output contracts.
 * @param args arguments with information about which rollups to connect
 * @param hre Hardhat Runtime Environment
 * @returns three connected contracts, Rollups, Input and Output
 */
export const connect = async (
    args: RollupsArgs,
    hre: HardhatRuntimeEnvironment
) => {
    const { ethers } = hre;
    const { RollupsImpl__factory, InputImpl__factory, OutputImpl__factory } =
        await import("../types");

    // choose a signer based on MNEMONIC and account index
    const signers = await ethers.getSigners();
    const index = args.accountIndex || 0;
    if (index < 0 || index >= signers.length) {
        throw new Error(
            `Invalid signer account index ${index}: must be between 0 and ${signers.length}`
        );
    }
    const signer = signers[index];

    // connect to RollupsImpl
    const rollupsContract = RollupsImpl__factory.connect(args.rollups, signer);

    // connect to InputImpl
    const inputContract = InputImpl__factory.connect(
        await rollupsContract.getInputAddress(),
        signer
    );

    // connect to OutputImpl
    const outputContract = OutputImpl__factory.connect(
        await rollupsContract.getOutputAddress(),
        signer
    );

    return {
        rollupsContract,
        inputContract,
        outputContract,
    };
};

type RollupsContracts = {
    rollupsContract: RollupsImpl;
    inputContract: InputImpl;
    outputContract: OutputImpl;
};

type RollupsAction<TArgs extends RollupsArgs> = (
    args: TArgs,
    rollups: RollupsContracts,
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
