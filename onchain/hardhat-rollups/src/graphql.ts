// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { Client, createClient, defaultExchanges } from "@urql/core";
import fetch from "cross-fetch";
import { ActionType, HardhatRuntimeEnvironment } from "hardhat/types";
import { GraphQLArgs } from "./args";

/**
 * Connects to a Rollups GraphQL server.
 * @param args arguments with information about which server to connect
 * @returns GraphQL client
 */
export const connect = (args: GraphQLArgs): Client => {
    // create GraphQL client to reader server
    return createClient({
        url: args.graphql,
        exchanges: defaultExchanges,
        fetch,
    });
};

type GraphQLAction<TArgs extends GraphQLArgs> = (
    args: TArgs,
    client: Client,
    hre: HardhatRuntimeEnvironment
) => Promise<any>;

/**
 * This is a wrapper around a hardhat task action that connects to Rollups
 * related contracts and calls a specialized action that deals with the
 * contracts.
 * @param action action that receives connected contracts
 * @returns harhat task action
 */
export function connected<TArgs extends GraphQLArgs>(
    action: GraphQLAction<TArgs>
): ActionType<TArgs> {
    return async (args, hre) => {
        // connect to rollups contracts
        const client = connect(args);

        // call the action
        return action(args, client, hre);
    };
}
