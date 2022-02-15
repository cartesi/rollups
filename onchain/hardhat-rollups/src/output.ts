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
import { ExecuteVoucherArgs } from "./args";
import { executeVoucherParams, rollupsParams } from "./params";
import { connected } from "./connect";
import { TASK_EXECUTE_VOUCHER } from "./constants";
import { taskDefs } from ".";

rollupsParams(
    executeVoucherParams(
        task<ExecuteVoucherArgs>(
            TASK_EXECUTE_VOUCHER,
            taskDefs[TASK_EXECUTE_VOUCHER].description,
            connected(async (args, { outputContract }) => {
                const signer = await outputContract.signer.getAddress();
                const proof = JSON.parse(args.proof); // string to JSON object

                const tx = await outputContract.executeVoucher(
                    args.destination,
                    args.payload,
                    proof
                )!;
                const events = (await tx.wait()).events!;
                const voucherExecutedEvent = getEvent(
                    "VoucherExecuted",
                    outputContract,
                    events
                );

                if (!voucherExecutedEvent) {
                    console.log(
                        `Failed to execute payload '${args.payload}' at destination '${args.destination}' with proof '${proof}' (signer: ${signer}, tx: ${tx.hash})`
                    );
                } else {
                    const voucherPosition =
                        voucherExecutedEvent.args.voucherPosition.toString();
                    console.log(
                        `Executed voucher at position '${voucherPosition}' (signer: ${signer}, tx: ${tx.hash})`
                    );
                }
            })
        )
    )
);
