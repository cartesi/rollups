import * as grpc from '@grpc/grpc-js';
import { DelegateManagerClient } from '../src/proto/stateserver_grpc_pb';

export default new DelegateManagerClient(
    `[::1]:50051`,
    grpc.credentials.createInsecure(),
);
