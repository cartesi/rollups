// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { TaskArguments } from "hardhat/types";

export interface CreateArgs extends TaskArguments {
    templateHash: string;
    inputDuration: number;
    challengePeriod: number;
    inputLog2Size: number;
    feePerClaim: number;
    erc20ForFee: string;
    feeManagerOwner: string;
    validators: string;
    log?: boolean;
}

export interface RollupsArgs extends TaskArguments {
    rollups: string;
    accountIndex?: number;
}

export interface ClaimArgs extends RollupsArgs {
    claim: string;
}

export interface AddInputArgs extends RollupsArgs {
    input: string;
}

export interface ExecuteVoucherArgs extends RollupsArgs {
    destination: string;
    payload: string;
    proof: string;
}

export interface AdvanceTimeArgs extends TaskArguments {
    seconds: number;
}
