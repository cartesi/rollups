// Copyright (C) 2022 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import client from './client';
import { GetStateRequest } from '../src/proto/stateserver_pb'
import { keccak256, defaultAbiCoder } from "ethers/lib/utils";

// Calculate input hash based on
// input: data itself interpreted by L2
// blockNumber: `block.number'
// blockTimestamp: `block.timestamp'
// epochIndex: epoch index
// inputIndex: input index
export const getInputHash = (input: any,
                             sender: string,
                             blockNumber: number,
                             blockTimestamp: number,
                             epochIndex: number,
                             inputIndex: number) => {

    // combine input attributes into one
    const metadata = defaultAbiCoder.encode(
        ["uint", "uint", "uint", "uint", "uint"],
        [sender, blockNumber, blockTimestamp, epochIndex, inputIndex]
    );

    // keccak the metadata and the input
    const keccak_metadata = keccak256(metadata);
    const keccak_input = keccak256(input);

    // combine the two keccaks into one
    const abi_metadata_input = defaultAbiCoder.encode(
        ["uint", "uint"],
        [keccak_metadata, keccak_input]
    );

    // keccak the combined keccaks
    const input_hash = keccak256(abi_metadata_input);

    // return the input hash
    return input_hash;
};

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
