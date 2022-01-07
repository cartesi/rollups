import client from "./client";
import { GetStateRequest } from "../generated-src/proto/stateserver_pb";

export const getState = async (initialState: string) => {
    const request = new GetStateRequest();
    request.setJsonInitialState(initialState);

    return new Promise<string>((resolve, reject) => {
        client.getState(request, (err, response) => {
            if (err) {
                return reject(err);
            }
            return resolve(response.getJsonState());
        });
    });
};
