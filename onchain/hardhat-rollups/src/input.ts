// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { task } from "hardhat/config";
import { getEvent } from "./eventUtil";
import { connected } from "./connect";
import { AddInputArgs } from "./args";
import { addInputParams, rollupsParams } from "./params";
import { TASK_ADD_INPUT, taskDefs } from "./constants";
import { ethers } from "ethers";
import { BytesLike } from "ethers";

rollupsParams(
    addInputParams(
        task<AddInputArgs>(
            TASK_ADD_INPUT,
            taskDefs[TASK_ADD_INPUT].description,
            connected(async (args, { inputFacet }) => {
                let inputBytes: BytesLike = args.input;
                if (!args.input.startsWith("0x")) {
                    // if input is a regular string (not a hex string), converts it to bytes assuming UTF-8
                    inputBytes = ethers.utils.toUtf8Bytes(args.input);
                }
                const signer = await inputFacet.signer.getAddress();
                const tx = await inputFacet.addInput(inputBytes);
                const events = (await tx.wait()).events ?? [];
                const inputAddedEvent = getEvent(
                    "InputAdded",
                    inputFacet,
                    events
                );
                if (!inputAddedEvent) {
                    console.log(
                        `Failed to add input '${args.input}' (signer: ${signer}, tx: ${tx.hash})`
                    );
                } else {
                    const epochNumber =
                        inputAddedEvent.args.epochNumber.toString();
                    const inputIndex =
                        inputAddedEvent.args.inputIndex.toString();
                    const timestamp = inputAddedEvent.args.timestamp.toString();
                    console.log(
                        `Added input '${args.input}' to epoch '${epochNumber}' (timestamp: ${timestamp}, signer: ${signer}, tx: ${tx.hash}, index: ${inputIndex})`
                    );
                }
            })
        )
    )
);
