// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import {
    accountIndexParam,
    addInputParams,
    claimParams,
    createParams,
    executeVoucherParams,
    fundBankParams,
    ParamsBuilder,
} from "./params";

type TaskDefinition = {
    description: string;
    params: ParamsBuilder;
};

export const TASK_CREATE = "rollups:create";
export const TASK_CLAIM = "rollups:claim";
export const TASK_FINALIZE_EPOCH = "rollups:finalizeEpoch";
export const TASK_GET_STATE = "rollups:getState";
export const TASK_ADD_INPUT = "rollups:addInput";
export const TASK_EXECUTE_VOUCHER = "rollups:executeVoucher";
export const TASK_FUND_BANK = "rollups:fundBank";

export const taskDefs: Record<string, TaskDefinition> = {
    [TASK_CREATE]: {
        description: "Create a Rollups diamond contract",
        params: createParams,
    },
    [TASK_CLAIM]: {
        description: "Send a claim to the current epoch",
        params: claimParams,
    },
    [TASK_FINALIZE_EPOCH]: {
        description: "Finalizes epoch, if challenge period has passed",
        params: accountIndexParam,
    },
    [TASK_GET_STATE]: {
        description: "Prints current epoch, current phase, input duration etc",
        params: accountIndexParam,
    },
    [TASK_ADD_INPUT]: {
        description: "Send an input to rollups",
        params: addInputParams,
    },
    [TASK_EXECUTE_VOUCHER]: {
        description: "Execute a voucher",
        params: executeVoucherParams,
    },
    [TASK_FUND_BANK]: {
        description: "Fund DApp's bank in order to pay validators",
        params: fundBankParams,
    },
};
