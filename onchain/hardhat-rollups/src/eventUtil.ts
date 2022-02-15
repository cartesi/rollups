// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { LogDescription } from "@ethersproject/abi";
import { ethers } from "ethers";

/**
 * Attempts to retrieve the first instance of the specified event from a given event logs array
 * @param eventName name/type of the event being retrieved
 * @param parser contract instance used to parse the event logs
 * @param eventLogs array of event logs (e.g., returned from a transaction)
 * @returns the expected parsed event log, or undefined if no corresponding event was found
 */
export function getEvent(
    eventName: string,
    parser: ethers.Contract,
    eventLogs: Array<any>
): LogDescription | undefined {
    let expectedEvent: LogDescription;
    for (let i = 0; i < eventLogs.length; i++) {
        try {
            expectedEvent = parser.interface.parseLog(eventLogs[i]);
            if (expectedEvent.name == eventName) {
                return expectedEvent;
            }
        } catch (e) {
            // do nothing, just skip to try parsing the next event
        }
    }
    // no corresponding event found
    return undefined;
}
