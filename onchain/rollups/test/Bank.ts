// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { deployments, ethers } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import {
    Bank,
    Bank__factory,
    SimpleToken,
    SimpleToken__factory,
} from "../src/types";
import { deployTestBank } from "./utils";

use(solidity);

describe("Bank", () => {
    let alice: Signer;
    let bob: Signer;
    let bank: Bank;
    let token: SimpleToken;
    let mockToken: MockContract;

    const initialSupply = 1000000;
    const addressZero = "0x0000000000000000000000000000000000000000";

    async function setupMockToken() {
        const IERC20 = await deployments.getArtifact("IERC20");
        mockToken = await deployMockContract(alice, IERC20.abi);
        let Bank = await deployments.deploy("Bank", {
            from: await alice.getAddress(),
            args: [mockToken.address],
        });
        bank = Bank__factory.connect(Bank.address, alice);
    }

    async function getState() {
        return {
            bankTokenBalance: await token.balanceOf(bank.address),
            aliceTokenBalance: await token.balanceOf(await alice.getAddress()),
            bobTokenBalance: await token.balanceOf(await bob.getAddress()),
            aliceBankBalance: await bank.balanceOf(await alice.getAddress()),
            bobBankBalance: await bank.balanceOf(await bob.getAddress()),
        };
    }

    beforeEach(async () => {
        await deployments.fixture();

        [alice, bob] = await ethers.getSigners();

        // Deploy a simple token
        let { Bank, SimpleToken } = await deployTestBank({ initialSupply });

        // Connect to token as the contract deployer (Alice)
        token = SimpleToken__factory.connect(SimpleToken.address, alice);

        // Connect to bank as Alice
        bank = Bank__factory.connect(Bank.address, alice);
    });

    it("Get token", async () => {
        expect(await bank.getToken(), "Get Bank token").to.equal(token.address);
    });

    it("Deploy bank with invalid token", async () => {
        await expect(
            deployments.deploy("Bank", {
                from: await alice.getAddress(),
                args: [addressZero],
            }),
            "Deploying Bank with address(0) as token should revert"
        ).to.be.revertedWith("Bank: invalid token");
    });

    it("Check initial state", async () => {
        const st = await getState();
        expect(
            st.bankTokenBalance,
            "Initial Bank's token balance should be zero"
        ).to.equal(0);
        expect(
            st.aliceTokenBalance,
            "Initial Alice's token balance should be `initialSupply`"
        ).to.equal(initialSupply);
        expect(
            st.bobTokenBalance,
            "Initial Bob's token balance should be zero"
        ).to.equal(0);
        expect(
            st.aliceBankBalance,
            "Initial Alice's bank balance should be zero"
        ).to.equal(0);
        expect(
            st.bobBankBalance,
            "Initial Bob's bank balance should be zero"
        ).to.equal(0);
    });

    it("Invalid recipient", async () => {
        await expect(
            bank.depositTokens(addressZero, 10),
            "Deposits to address(0) should revert"
        ).to.be.revertedWith("Bank: invalid recipient");
    });

    it("Deposit 0 tokens", async () => {
        const st1 = await getState();

        await expect(
            bank.depositTokens(await bob.getAddress(), 0),
            "Deposit 0 tokens in Bob's account"
        )
            .to.emit(bank, "Deposit")
            .withArgs(await alice.getAddress(), await bob.getAddress(), 0);

        const st2 = await getState();

        expect(
            st2.bankTokenBalance,
            "Bank's token balance should be the same"
        ).to.equal(st1.bankTokenBalance);
        expect(
            st2.aliceTokenBalance,
            "Alice's token balance should be the same"
        ).to.equal(st1.aliceTokenBalance);
        expect(
            st2.bobTokenBalance,
            "Bob's token balance should be the same"
        ).to.equal(st1.bobTokenBalance);
        expect(
            st2.aliceBankBalance,
            "Alice's bank balance should be the same"
        ).to.equal(st1.aliceBankBalance);
        expect(
            st2.bobBankBalance,
            "Bob's bank balance should be the same"
        ).to.equal(st1.bobBankBalance);
    });

    it("Not enough approval", async () => {
        // Give no approval
        await expect(
            bank.depositTokens(await bob.getAddress(), 10),
            "Deposits with not enough approval should revert"
        ).to.be.revertedWith("ERC20: insufficient allowance");

        // Give insufficient approval
        await token.approve(bank.address, 9);
        await expect(
            bank.depositTokens(await bob.getAddress(), 10),
            "Deposits with not enough approval should revert"
        ).to.be.revertedWith("ERC20: insufficient allowance");

        // Give too much approval
        await token.approve(bank.address, initialSupply + 1);
        await expect(
            bank.depositTokens(await bob.getAddress(), initialSupply + 1),
            "Deposits with not enough balance should revert"
        ).to.be.revertedWith("ERC20: transfer amount exceeds balance");

        // Have no balance
        await token.connect(bob).approve(bank.address, 10);
        await expect(
            bank.connect(bob).depositTokens(await alice.getAddress(), 10),
            "Deposits with not enough balance should revert"
        ).to.be.revertedWith("ERC20: transfer amount exceeds balance");
    });

    it("Just enough approval", async () => {
        const value = 10;

        const st1 = await getState();

        // Deposits `value` tokens into Bank
        await token.approve(bank.address, value);
        expect(
            await bank.depositTokens(await bob.getAddress(), value),
            "A successful call to depositTokens should emit a Deposit event"
        )
            .to.emit(bank, "Deposit")
            .withArgs(await alice.getAddress(), await bob.getAddress(), value);

        const st2 = await getState();

        expect(
            st2.bankTokenBalance,
            "Bank's token balance should be increased by `value`"
        ).to.equal(st1.bankTokenBalance.add(value));
        expect(
            st2.aliceTokenBalance,
            "Alice's token balance should be decreased by `value`"
        ).to.equal(st1.aliceTokenBalance.sub(value));
        expect(
            st2.bobTokenBalance,
            "Bob's token balance should be the same"
        ).to.equal(st1.bobTokenBalance);
        expect(
            st2.aliceBankBalance,
            "Alice's bank balance should be the same"
        ).to.equal(st1.aliceBankBalance);
        expect(
            st2.bobBankBalance,
            "Bob's bank balance should be increased by `value`"
        ).to.equal(st1.bobBankBalance.add(value));

        // Approved amount should be updated
        await expect(
            bank.depositTokens(await bob.getAddress(), 1),
            "Deposits with not enough approval should revert"
        ).to.be.revertedWith("ERC20: insufficient allowance");
    });

    it("Transfer 0 tokens", async () => {
        const st1 = await getState();

        await expect(
            bank.transferTokens(await bob.getAddress(), 0),
            "Transfer 0 tokens to Bob's account"
        )
            .to.emit(bank, "Transfer")
            .withArgs(await alice.getAddress(), await bob.getAddress(), 0);

        const st2 = await getState();

        expect(
            st2.bankTokenBalance,
            "Bank's token balance should be the same"
        ).to.equal(st1.bankTokenBalance);
        expect(
            st2.aliceTokenBalance,
            "Alice's token balance should be the same"
        ).to.equal(st1.aliceTokenBalance);
        expect(
            st2.bobTokenBalance,
            "Bob's token balance should be the same"
        ).to.equal(st1.bobTokenBalance);
        expect(
            st2.aliceBankBalance,
            "Alice's bank balance should be the same"
        ).to.equal(st1.aliceBankBalance);
        expect(
            st2.bobBankBalance,
            "Bob's bank balance should be the same"
        ).to.equal(st1.bobBankBalance);
    });

    it("Failed transferFrom", async () => {
        await setupMockToken();
        await mockToken.mock.transferFrom.returns(false);
        await expect(
            bank.depositTokens(await bob.getAddress(), 0),
            "Depositing 0 tokens in Bob's bank account"
        ).to.be.revertedWith("Bank: transferFrom failed");
    });

    it("Not enough balance", async () => {
        let st = await getState();

        // Try transfering with no balance
        await expect(
            bank.transferTokens(
                await bob.getAddress(),
                st.aliceBankBalance.add(1)
            ),
            "Transfering too many token"
        ).to.be.revertedWith("Bank: not enough balance");

        // Deposit some tokens into Alice's bank account
        const value = 10;
        await token.approve(bank.address, value);
        await bank.depositTokens(await alice.getAddress(), value);

        // Get new state
        st = await getState();

        // Try transfering with some balance
        await expect(
            bank.transferTokens(
                await bob.getAddress(),
                st.aliceBankBalance.add(1)
            ),
            "Transfering too many token"
        ).to.be.revertedWith("Bank: not enough balance");
    });

    it("Just enough balance", async () => {
        const value = 10;
        await token.approve(bank.address, value);
        await bank.depositTokens(await alice.getAddress(), value);

        const st1 = await getState();

        await expect(
            bank.transferTokens(await bob.getAddress(), value),
            `Transfering ${value} tokens to Bob's token account`
        )
            .to.emit(bank, "Transfer")
            .withArgs(await alice.getAddress(), await bob.getAddress(), value);

        const st2 = await getState();

        expect(
            st2.bankTokenBalance,
            "Bank's token balance should decreased by `value`"
        ).to.equal(st1.bankTokenBalance.sub(value));
        expect(
            st2.aliceTokenBalance,
            "Alice's token balance should be the same"
        ).to.equal(st1.aliceTokenBalance);
        expect(
            st2.bobTokenBalance,
            "Bob's token balance should be increased by `value`"
        ).to.equal(st1.bobTokenBalance.add(value));

        expect(
            st2.aliceBankBalance,
            "Alice's bank balance should be decreased by `value`"
        ).to.equal(st1.aliceBankBalance.sub(value));
        expect(
            st2.bobBankBalance,
            "Bob's bank balance should be the same"
        ).to.equal(st1.bobBankBalance);
    });

    it("Failed transfer", async () => {
        await setupMockToken();
        await mockToken.mock.transfer.returns(false);
        await expect(
            bank.transferTokens(await bob.getAddress(), 0),
            "Depositing 0 tokens in Bob's bank account"
        ).to.be.revertedWith("Bank: transfer failed");
    });

    it("Balance overflow", async () => {
        await setupMockToken();
        await mockToken.mock.transferFrom.returns(true);

        const value = ethers.constants.MaxUint256;

        await expect(
            bank.depositTokens(await bob.getAddress(), value),
            "Deposit `uint256.max` tokens in Bob's bank account"
        )
            .to.emit(bank, "Deposit")
            .withArgs(await alice.getAddress(), await bob.getAddress(), value);

        await expect(
            bank.depositTokens(await bob.getAddress(), 1),
            "Deposit 1 token in Bob's bank account"
        ).to.be.reverted;
    });
});
