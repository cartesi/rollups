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
import "hardhat-deploy/dist/src/type-extensions";

import {
    taskDefs,
    TASK_ADD_INPUT,
    TASK_CLAIM,
    TASK_EXECUTE_VOUCHER,
    TASK_FINALIZE_EPOCH,
    TASK_FUND_BANK,
    TASK_GET_STATE,
    TASK_GET_NOTICES,
} from "./constants";

export type GraphQLConfig = Record<string, string>;

/**
 * Action wrapper that resolves the Rollups contract address from DApp deployment and injects
 * the appropriate graphql endpoint
 * @param taskName name of the task to be executed
 * @returns whatever the task returns
 */
const rollupsAction = (
    taskName: string,
    graphqlConfig: GraphQLConfig
): ActionType<any> => {
    return async (args: any, hre: HardhatRuntimeEnvironment) => {
        const { deployments, network, run } = hre;

        // retrieves GraphQL endpoint for the network being used
        let graphqlEndpoint;
        if (network.name in graphqlConfig) {
            graphqlEndpoint = graphqlConfig[network.name];
        }

        return run(taskName, {
            graphql: graphqlEndpoint,
            ...args,
        });
    };
};

/**
 * Create application-specific tasks that call generic rollups tasks within
 * the scope of a deployed rollups contract
 * @param appName name of rollups application
 */
export const appTasks = (appName: string, graphqlConfig: GraphQLConfig) => {
    [
        TASK_CLAIM,
        TASK_FINALIZE_EPOCH,
        TASK_FUND_BANK,
        TASK_GET_STATE,
        TASK_ADD_INPUT,
        TASK_GET_NOTICES,
        TASK_EXECUTE_VOUCHER,
    ].forEach((taskName) => {
        const taskDef = taskDefs[taskName];

        // define app task name, i.e. echo:claim -> rollups:claim
        const newTaskName = taskName.replace(/^rollups/, appName);

        // define new task
        taskDef.params(
            task(
                newTaskName,
                taskDef.description,
                rollupsAction(taskName, graphqlConfig)
            )
        );
    });
};
