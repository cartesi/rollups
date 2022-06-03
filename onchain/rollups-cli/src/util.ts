// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

type Handler<U> = (args: U) => void | Promise<void>;

/**
 * Create a wrapper handler that will catch any errors, print them to stderr, and exit with a non-zero code.
 * @param handler unsafe handler
 * @returns wrapper function that handles errors gracefully
 */
export const safeHandler =
    <U>(handler: Handler<U>): Handler<U> =>
    async (args) => {
        try {
            await handler(args);
        } catch (e: any) {
            console.error(e.message);
            process.exit(1);
        }
    };
