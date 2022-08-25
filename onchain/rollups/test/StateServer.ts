// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import path from "path";
import fs from "fs";
import * as protoLoader from "@grpc/proto-loader";
import * as grpc from "@grpc/grpc-js";
import { ProtoGrpcType } from "../generated-src/proto/state-fold-server";
import { StateFoldClient } from "../generated-src/proto/StateFoldServer/StateFold";
import { BlockState__Output } from "../generated-src/proto/StateFoldServer/BlockState";

const BINARIES_PATH =
    process.env.BINARIES_PATH || "../../offchain/target/debug/";

type FOLD =
    | "input"
    | "output"
    | "validator_manager"
    | "fee_manager"
    | "rollups";

const ports: Record<FOLD, number> = {
    input: 50051,
    output: 50052,
    validator_manager: 50053,
    fee_manager: 50054,
    rollups: 50055,
};

const binaries: Record<FOLD, string> = {
    input: "input_state_server",
    output: "output_state_server",
    validator_manager: "validator_manager_state_server",
    fee_manager: "fee_manager_state_server",
    rollups: "rollups_state_server",
};

const createClient = (address: string): StateFoldClient => {
    // load proto definition
    const packageDefinition = protoLoader.loadSync(
        "../../grpc-interfaces/state-fold-server.proto"
    );

    // turn into proto object
    const proto = grpc.loadPackageDefinition(
        packageDefinition
    ) as unknown as ProtoGrpcType;

    // create client
    return new proto.StateFoldServer.StateFold(
        address,
        grpc.credentials.createInsecure()
    );
};

const getClientState = async (
    client: StateFoldClient,
    jsonData: string
): Promise<string> => {
    return new Promise<string>((resolve, reject) => {
        const initialState = { jsonData };
        client.QueryState(
            { initialState, queryBlock: null },
            (
                err: grpc.ServiceError | null,
                response: BlockState__Output | undefined
            ) => {
                if (err || !response?.state?.jsonData) {
                    return reject(err ?? `no response`);
                }
                return resolve(response.state.jsonData);
            }
        );
    });
};

const logFile = (fullpath: string): number => {
    const directory = path.dirname(fullpath);
    if (!fs.existsSync(directory)) {
        fs.mkdirSync(directory, { recursive: true });
    }
    return fs.openSync(fullpath, "w");
};

const spawn_server = function (fold: FOLD) {
    // only spawn state server is running against network at localhost (not on hardhat)
    if (process.env.STATE_FOLD_TEST) {
        const port = ports[fold];
        const stdout = logFile(`logs/${fold}.stdout.log`);
        const stderr = logFile(`logs/${fold}.stderr.log`);
        const process = spawn(
            path.join(BINARIES_PATH, binaries[fold]),
            [
                "--sf-safety-margin",
                "0",
                "--ss-server-address",
                `0.0.0.0:${port}`,
            ],
            {
                stdio: [null, stdout, stderr],
                env: {
                    RUST_LOG: "INFO",
                },
            }
        );
        const client = createClient(`127.0.0.1:${port}`);
        const getState = async (jsonData: string): Promise<string> =>
            getClientState(client, jsonData);

        return {
            process,
            getState,
        };
    }
};

export const input: Mocha.AsyncFunc = async function () {
    const server = spawn_server("input");
    this.process = server?.process;
    this.getState = server?.getState;
};

export const output: Mocha.AsyncFunc = async function () {
    const server = spawn_server("output");
    this.process = server?.process;
    this.getState = server?.getState;
};

export const validator_manager: Mocha.AsyncFunc = async function () {
    const server = spawn_server("validator_manager");
    this.process = server?.process;
    this.getState = server?.getState;
};

export const fee_manager: Mocha.AsyncFunc = async function () {
    const server = spawn_server("fee_manager");
    this.process = server?.process;
    this.getState = server?.getState;
};

export const rollups: Mocha.AsyncFunc = async function () {
    const server = spawn_server("rollups");
    this.process = server?.process;
    this.getState = server?.getState;
};

export const kill: Mocha.Func = function () {
    if (this.process) {
        const process = this.process as ChildProcessWithoutNullStreams;
        process.kill();
    }
};

export interface StateServerContext extends Mocha.Context {
    process: ChildProcessWithoutNullStreams;
    getState: (jsonData: string) => Promise<string>;
}
