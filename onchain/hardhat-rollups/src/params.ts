// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { types } from "hardhat/config";
import { ConfigurableTaskDefinition } from "hardhat/types";

export type ParamsBuilder = (
    task: ConfigurableTaskDefinition
) => ConfigurableTaskDefinition;

export const createParams: ParamsBuilder = (task) => {
    return task
        .addOptionalParam<boolean>(
            "log",
            "Enable log output",
            true,
            types.boolean
        )
        .addParam<string>(
            "templateHash",
            "Template hash of the machine",
            undefined,
            types.string,
            true
        )
        .addParam<number>(
            "inputDuration",
            "Time window of input collection, in seconds",
            86400,
            types.int,
            true
        )
        .addParam<number>(
            "challengePeriod",
            "Time window of challenge, in seconds",
            604800,
            types.int,
            true
        )
        .addParam<number>(
            "inputLog2Size",
            "Log2 size of input",
            25,
            types.int,
            true
        )
        .addParam<number>(
            "feePerClaim",
            "Fee to reward validators for claims",
            10,
            types.int,
            true
        )
        .addParam<string>(
            "erc20ForFee",
            "Address of ERC-20 token used to reward validators",
            undefined,
            types.string,
            true
        )
        .addParam<number>(
            "feeManagerOwner",
            "Address of Fee Manager owner. Defaults to the address of the deployer.",
            undefined,
            types.string,
            true
        )
        .addParam<string>(
            "validators",
            "Comma separated list of validator nodes addresses. If item is a number consider as an account index of the defined MNEMONIC",
            "0,1,2",
            types.string,
            true
        );
};

export const accountIndexParam: ParamsBuilder = (task) => {
    return task.addOptionalParam<number>(
        "accountIndex",
        "Account index of the signer from defined MNEMONIC",
        0,
        types.int
    );
};

export const rollupsParams: ParamsBuilder = (task) => {
    return task.addParam<string>(
        "rollups",
        "Address of rollups contract",
        undefined,
        types.string,
        false
    );
};

export const claimParams: ParamsBuilder = (task) => {
    return accountIndexParam(task).addParam<string>(
        "claim",
        "Validator's bytes32 claim for current claimable epoch",
        undefined,
        types.string,
        false
    );
};

export const addInputParams: ParamsBuilder = (task) => {
    return accountIndexParam(task).addParam<string>(
        "input",
        "Bytes to be processed by the offchain machine",
        undefined,
        types.string
    );
};

export const executeVoucherParams: ParamsBuilder = (task) => {
    return accountIndexParam(task)
        .addParam(
            "destination",
            "The destination address that is called for execution"
        )
        .addParam(
            "payload",
            "The abi encoding of the called function and arguments"
        )
        .addParam(
            "proof",
            "Proof for the voucher being executed. Should be wrapped as a JSON string."
        );
};

export const advanceTimeParams: ParamsBuilder = (task) => {
    return task.addParam("seconds", "Time to advance in seconds");
};
