#!/usr/bin/env node
// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import create from "./commands/create";

yargs(hideBin(process.argv))
    .version()
    .command(create)
    .strict()
    .alias({ h: "help" }).argv;
