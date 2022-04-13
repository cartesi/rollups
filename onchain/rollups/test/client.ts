// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import * as protoLoader from "@grpc/proto-loader";
import * as grpc from "@grpc/grpc-js";

import { ProtoGrpcType } from "../generated-src/proto/stateserver";

const createClient = (address: string = "0.0.0.0:50051") => {
    // load proto definition
    const packageDefinition = protoLoader.loadSync(
        "../../grpc-interfaces/stateserver.proto"
    );

    // turn into proto object
    const proto = grpc.loadPackageDefinition(
        packageDefinition
    ) as unknown as ProtoGrpcType;

    // create client
    return new proto.StateServer.DelegateManager(
        address,
        grpc.credentials.createInsecure()
    );
};

export default createClient;
