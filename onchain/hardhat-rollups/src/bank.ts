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

                // query balance
                const signerBalance = await token.balanceOf(signerAddress);
                const bankBalance = await token.balanceOf(bankAddress);
                console.log(
                    `user balance(${signerAddress}): ${signerBalance.toString()}`
                );
                console.log(
                    `bank balance(${bankAddress}): ${bankBalance.toString()}`
                );

                if (amount.gt(signerBalance)) {
                    throw Error(
                        `not enough balance for account ${signerAddress}: ${signerBalance.toString()}`
                    );
                }

                // query current allowance
                const currentAllowance = await token.allowance(
                    signerAddress,
                    bankAddress
                );
                console.log(
                    `allowance(${signerAddress},${bankAddress}): ${currentAllowance.toString()}`
                );

                if (amount.gt(currentAllowance)) {
                    // Allow bank to withdraw `amount` tokens from signer
                    const tx = await token.approve(bankAddress, amount);
                    const receipt = await tx.wait(1);
                    const event = (
                        await token.queryFilter(
                            token.filters.Approval(),
                            receipt.blockHash
                        )
                    ).pop();
                    if (!event) {
                        throw Error(
                            `could not approve ${amount} tokens for DApp(${feeManagerFacet.address})'s bank (signer: ${signerAddress}, tx: ${tx.hash})`
                        );
                    } else {
                        console.log(
                            `approved ${amount} tokens for DApp(${feeManagerFacet.address})'s bank (signer: ${signerAddress}, tx: ${tx.hash})`
                        );
                    }
                }

                // Transfer `amount` tokens to bank and increase DApp's balance in bank by `amount`
                const tx = await bank.depositTokens(
                    feeManagerFacet.address,
                    amount
                );
                const receipt = await tx.wait(1);
                const event = (
                    await bank.queryFilter(
                        bank.filters.Deposit(),
                        receipt.blockHash
                    )
                ).pop();

                if (!event) {
                    throw Error(
                        `failed to fund DApp(${feeManagerFacet.address})'s bank with ${amount} tokens (signer: ${signerAddress}, tx: ${tx.hash})`
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
