import * as grpc from "@grpc/grpc-js";
import { DelegateManagerClient } from "../generated-src/proto/stateserver_grpc_pb";

export default new DelegateManagerClient(
    `0.0.0.0:50051`,
    grpc.credentials.createInsecure()
);
