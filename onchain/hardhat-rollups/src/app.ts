// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { task } from "hardhat/config";
import { ActionType, HardhatRuntimeEnvironment } from "hardhat/types";
import {
    taskDefs,
    TASK_ADD_INPUT,
    TASK_CLAIM,
    TASK_EXECUTE_VOUCHER,
    TASK_FINALIZE_EPOCH,
    TASK_GET_STATE,
} from "./constants";

/**
 * Action wrapper that resolves the Rollups contract address from DApp deployment
 * @param taskName name of the task to be executed
 * @returns whatever the task returns
 */
const rollupsAction = (taskName: string): ActionType<any> => {
    return async (args: any, hre: HardhatRuntimeEnvironment) => {
        const { deployments, run } = hre;
        const Rollups = await deployments.get("RollupsImpl");
        return run(taskName, { rollups: Rollups.address, ...args });
    };
};

/**
 * Create application-specific tasks that call generic rollups tasks within
 * the scope of a deployed rollups contract
 * @param appName name of rollups application
 */
export const appTasks = (appName: string) => {
    [
        TASK_CLAIM,
        TASK_FINALIZE_EPOCH,
        TASK_GET_STATE,
        TASK_ADD_INPUT,
        TASK_EXECUTE_VOUCHER,
    ].forEach((taskName) => {
        const taskDef = taskDefs[taskName];

        // define app task name, i.e. echo:claim -> rollups:claim
        const newTaskName = taskName.replace(/^rollups/, appName);

        // define new task
        taskDef.params(
            task(newTaskName, taskDef.description, rollupsAction(taskName))
        );
    });
};
