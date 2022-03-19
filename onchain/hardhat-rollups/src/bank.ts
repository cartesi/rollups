// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { task } from "hardhat/config";
import { getEvent } from "./eventUtil";
import { FundBankArgs } from "./args";
import { fundBankParams, rollupsParams } from "./params";
import { connected } from "./connect";
import { TASK_FUND_BANK } from "./constants";
import { taskDefs } from ".";
import { BigNumber } from "ethers";
import { Bank__factory, IERC20__factory } from "@cartesi/rollups";

rollupsParams(
    fundBankParams(
        task<FundBankArgs>(
            TASK_FUND_BANK,
            taskDefs[TASK_FUND_BANK].description,
            connected(async (args, { feeManagerFacet }) => {
                const signer = feeManagerFacet.signer;
                const signerAddress = await signer.getAddress();
                const amount = BigNumber.from(args.amount);

                const bankAddress = await feeManagerFacet.getFeeManagerBank();
                const bank = Bank__factory.connect(bankAddress, signer);

                const tokenAddress = await bank.getToken();
                const token = IERC20__factory.connect(tokenAddress, signer);

                let tx;
                let events;

                // Allow bank to withdraw `amount` tokens from signer
                tx = await token.approve(bankAddress, amount);
                events = (await tx.wait()).events ?? [];
                const approvalEvent = getEvent("Approval", token, events);
                if (!approvalEvent) {
                    throw Error(
                        `Could not approve ${amount} tokens for DApp(${feeManagerFacet.address})'s bank (signer: ${signerAddress}, tx: ${tx.hash})`
                    );
                } else {
                    console.log(
                        `Approved ${amount} tokens for DApp(${feeManagerFacet.address})'s bank (signer: ${signerAddress}, tx: ${tx.hash})`
                    );
                }

                // Transfer `amount` tokens to bank and increase DApp's balance in bank by `amount`
                tx = await bank.depositTokens(feeManagerFacet.address, amount);
                events = (await tx.wait()).events ?? [];
                const depositEvent = getEvent(
                    "Deposit",
                    feeManagerFacet,
                    events
                );

                if (!depositEvent) {
                    throw Error(
                        `Failed to fund DApp(${feeManagerFacet.address})'s bank with ${amount} tokens (signer: ${signerAddress}, tx: ${tx.hash})`
                    );
                } else {
                    console.log(
                        `funded DApp(${feeManagerFacet.address})'s bank with ${amount} tokens (signer: ${signerAddress}, tx: ${tx.hash})`
                    );
                }
            })
        )
    )
);
