// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

import { ArgumentsCamelCase } from "yargs";

type Handler<U> = (args: ArgumentsCamelCase<U>) => void | Promise<void>;

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
