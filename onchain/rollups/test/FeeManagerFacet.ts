import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { RollupsFacet } from "../src/types/RollupsFacet";
import { RollupsFacet__factory } from "../src/types/factories/RollupsFacet__factory";
import { FeeManagerFacet } from "../src/types/FeeManagerFacet";
import { FeeManagerFacet__factory } from "../src/types/factories/FeeManagerFacet__factory";
import { Bank } from "../src/types/Bank";
import { Bank__factory } from "../src/types/factories/Bank__factory";
import { DebugFacet } from "../src/types/DebugFacet";
import { DebugFacet__factory } from "../src/types/factories/DebugFacet__factory";
import { SimpleToken } from "../src/types/SimpleToken";
import { SimpleToken__factory } from "../src/types/factories/SimpleToken__factory";
import { DiamondInit } from "../src/types/DiamondInit";
import { DiamondInit__factory } from "../src/types/factories/DiamondInit__factory";
import { ERC20PortalFacet } from "../src/types/ERC20PortalFacet";
import { ERC20PortalFacet__factory } from "../src/types/factories/ERC20PortalFacet__factory";
import { deployDiamond, getState, increaseTimeAndMine } from "./utils";

use(solidity);

describe("FeeManager Facet", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let signers: Signer[];
    let token: SimpleToken;
    let feeManagerFacet: FeeManagerFacet;
    let bank: Bank;
    let rollupsFacet: RollupsFacet;
    let diamondInit: DiamondInit;
    let debugFacet: DebugFacet;
    let portalFacet: ERC20PortalFacet;
    let tokenSupply = 1000000; // assume FeeManagerImpl contract owner has 1 million tokens (ignore decimals)
    let initialFeePerClaim = 10; // set initial fees per claim as 10 token
    let initialState: string; // for delegate
    let inputDuration: number;
    let challengePeriod: number;

    // Increase the current time in the network by just above
    // the input duration and force a block to be mined
    async function passInputAccumulationPeriod() {
        await increaseTimeAndMine(inputDuration + 1);
    }

    // Increase the current time in the network by just above
    // the challenge period and force a block to be mined
    async function passChallengePeriod() {
        await increaseTimeAndMine(challengePeriod + 1);
    }

    // Deposits `amount` tokens in Fee Manager's bank account
    async function fundFeeManager(amount: number) {
        const [tokenOwner] = signers;
        const tokenOwnerAddress = await tokenOwner.getAddress();

        // get balance before funding
        const ownerTokenBalance = await token.balanceOf(tokenOwnerAddress);
        const bankTokenBalance = await token.balanceOf(bank.address);
        const dappBankBalance = await bank.balanceOf(feeManagerFacet.address);

        // approve bank to withdraw `amount` from tokenOwner's account
        expect(await token.approve(bank.address, amount))
            .to.emit(token, "Approval")
            .withArgs(tokenOwnerAddress, bank.address, amount);

        // deposit `amount` tokens in Fee Manager's bank account
        expect(await bank.depositTokens(feeManagerFacet.address, amount))
            .to.emit(bank, "Deposit")
            .withArgs(tokenOwnerAddress, feeManagerFacet.address, amount);

        // check balances after funding
        expect(
            await token.balanceOf(bank.address),
            "bank received `amount` tokens"
        ).to.equal(bankTokenBalance.add(amount));
        expect(
            await token.balanceOf(tokenOwnerAddress),
            "token owner deposited `amount` tokens"
        ).to.equal(ownerTokenBalance.sub(amount));
        expect(
            await bank.balanceOf(feeManagerFacet.address),
            "feeManager's balance increased by `amount`"
        ).to.equal(dappBankBalance.add(amount));
    }

    beforeEach(async () => {
        // get signers
        signers = await ethers.getSigners();

        const diamond = await deployDiamond({
            debug: true,
            simpleFeeManagerBank: true,
        });

        debugFacet = DebugFacet__factory.connect(diamond.address, signers[0]);

        feeManagerFacet = FeeManagerFacet__factory.connect(
            diamond.address,
            signers[0]
        );

        rollupsFacet = RollupsFacet__factory.connect(
            diamond.address,
            signers[0]
        );

        portalFacet = ERC20PortalFacet__factory.connect(
            diamond.address,
            signers[0]
        );

        diamondInit = DiamondInit__factory.connect(diamond.address, signers[0]);

        inputDuration = (await rollupsFacet.getInputDuration()).toNumber();
        challengePeriod = (await rollupsFacet.getChallengePeriod()).toNumber();

        const bankAddress = await feeManagerFacet.getFeeManagerBank();
        bank = Bank__factory.connect(bankAddress, signers[0]);

        const tokenAddress = await bank.getToken();
        token = SimpleToken__factory.connect(tokenAddress, signers[0]);

        // for delegate
        initialState = JSON.stringify({
            dapp_contract_address: feeManagerFacet.address,
        });
    });

    it("test initial feePerClaim", async () => {
        expect(
            await debugFacet._getFeePerClaim(),
            "initial feePerClaim"
        ).to.equal(initialFeePerClaim);
    });

    if (enableDelegate) {
        it("test delegate initial data", async () => {
            let state = JSON.parse(await getState(initialState));

            // dapp_contract_address
            expect(state.dapp_contract_address, "delegate address").to.equal(
                feeManagerFacet.address.toLowerCase()
            );
            // bank_address
            expect(state.bank_address, "bank address").to.equal(
                bank.address.toLowerCase()
            );
            // fee_per_claim
            same_as_initial_value("fee_per_claim", state);
            // validator_redeemed
            same_as_initial_value("validator_redeemed", state);
            // bank_balance
            same_as_initial_value("bank_balance", state);
            // uncommitted_balance
            same_as_initial_value("uncommitted_balance", state);
        });
    }

    it("test constructor event FeeManagerCreated", async () => {
        let eventFilter = diamondInit.filters.FeeManagerInitialized(
            null,
            null,
            null
        );
        let event = await diamondInit.queryFilter(eventFilter);
        let eventArgs = event[0]["args"];

        expect(eventArgs["feePerClaim"], "feePerClaim").to.equal(
            initialFeePerClaim
        );
        expect(
            eventArgs["feeManagerOwner"],
            "fee manager owner address"
        ).to.equal(await signers[0].getAddress());
        expect(
            eventArgs["feeManagerBank"],
            "fee manager bank address"
        ).to.equal(bank.address);
    });

    it("fund the FeeManager contract and emit event", async () => {
        expect(
            await token.balanceOf(bank.address),
            "initially the bank has no erc20 tokens"
        ).to.equal(0);

        // fund 10000 tokens
        let amount = 10000;
        await fundFeeManager(amount);

        // test delegate
        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));
            expect(
                parseInt(state.bank_balance, 16),
                "fee manager bank has 10k tokens deposited"
            ).to.equal(amount);
            expect(
                state.uncommitted_balance,
                "no claims yet, so uncommitted balance is the same as total balance"
            ).to.equal(amount);

            // the rest should stay the same
            same_as_initial_value("fee_per_claim", state);
            same_as_initial_value("validator_redeemed", state);
        }

        // again, fund 10000 tokens
        await fundFeeManager(amount);

        // test delegate
        if (enableDelegate) {
            let state = JSON.parse(await getState(initialState));
            expect(
                parseInt(state.bank_balance, 16),
                "fee manager bank has 20k tokens deposited"
            ).to.equal(amount * 2);
            expect(
                state.uncommitted_balance,
                "no claims yet, so uncommitted balance is the same as total balance"
            ).to.equal(amount * 2);

            // the rest should stay the same
            same_as_initial_value("fee_per_claim", state);
            same_as_initial_value("validator_redeemed", state);
        }
    });

    // test numClaimsRedeemable
    it("test numClaimsRedeemable()", async () => {
        // revert on address 0
        let address_zero = "0x0000000000000000000000000000000000000000";
        await expect(
            feeManagerFacet.numClaimsRedeemable(address_zero),
            "should revert on address 0"
        ).to.be.revertedWith("address should not be 0");

        // initially signers[1] has 0 redeemable claims
        expect(
            await feeManagerFacet.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "initially, no redeemable claims"
        ).to.equal(0);

        // assume signers[1] makes 10 claims
        await debugFacet._setNumClaims(1, 10);
        expect(
            await feeManagerFacet.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "now there are 10 claims redeemable"
        ).to.equal(10);

        // owner funds the FeeManager and signers[1] redeem fees
        await fundFeeManager(10000);
        await feeManagerFacet.redeemFee(await signers[1].getAddress());

        // after having redeemed, no more redeemable claims
        expect(
            await feeManagerFacet.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "redeemable claims are all redeemed"
        ).to.equal(0);

        // test default "underflow check" by sol0.8
        await debugFacet._setNumClaims(1, 0);
        await expect(
            feeManagerFacet.numClaimsRedeemable(await signers[1].getAddress()),
            "0 - 10"
        ).to.be.reverted;
    });

    // test getNumClaimsRedeemed
    it("test getNumClaimsRedeemed()", async () => {
        // revert on address 0
        let address_zero = "0x0000000000000000000000000000000000000000";
        await expect(
            feeManagerFacet.getNumClaimsRedeemed(address_zero),
            "getNumClaimsRedeemed() should revert on address 0"
        ).to.be.revertedWith("address should not be 0");

        // initially signers[1] has 0 redeemed claims
        expect(
            await feeManagerFacet.callStatic.getNumClaimsRedeemed(
                await signers[1].getAddress()
            ),
            "initially, no redeemed claims"
        ).to.equal(0);

        // assume signers[1] redeems 10 claims
        await debugFacet._setNumClaims(1, 10);
        await fundFeeManager(10000);
        await feeManagerFacet.redeemFee(await signers[1].getAddress());

        expect(
            await feeManagerFacet.callStatic.getNumClaimsRedeemed(
                await signers[1].getAddress()
            ),
            "now #redeemed should be 10"
        ).to.equal(10);
    });

    // redeem fees
    it("test redeemFee() when no claims have been made", async () => {
        // assume signers[1] makes 10 claims
        await expect(
            feeManagerFacet.redeemFee(await signers[1].getAddress()),
            "no claims made"
        ).to.be.revertedWith("nothing to redeem yet");
    });

    it("redeemFee on his/her own", async () => {
        if (!enableDelegate) {
            //owner funds FeeManager
            let amount = 10000;
            await fundFeeManager(amount);

            // assume signers[1] makes 10 claims
            await debugFacet._setNumClaims(1, 10);

            await expect(
                feeManagerFacet
                    .connect(signers[1])
                    .redeemFee(await signers[1].getAddress()),
                "redeem fee for 10 claims"
            )
                .to.emit(feeManagerFacet, "FeeRedeemed")
                .withArgs(await signers[1].getAddress(), 10);

            // check balances
            expect(
                await token.balanceOf(bank.address),
                "bank now has 10*initialFeePerClaim less tokens"
            ).to.equal(amount - 10 * initialFeePerClaim);
            expect(
                await bank.balanceOf(feeManagerFacet.address),
                "feeManager's bank balance got decreased by 10*initialFeePerClaim"
            ).to.equal(amount - 10 * initialFeePerClaim);
            expect(
                await token.balanceOf(await signers[1].getAddress()),
                "validator now has 10*initialFeePerClaim tokens"
            ).to.equal(10 * initialFeePerClaim);

            // redeemFee again
            await expect(
                feeManagerFacet
                    .connect(signers[1])
                    .redeemFee(await signers[1].getAddress()),
                "no additional claims made"
            ).to.be.revertedWith("nothing to redeem yet");

            // make more claims and then redeemFee
            await debugFacet._setNumClaims(1, 30);
            await expect(
                feeManagerFacet
                    .connect(signers[1])
                    .redeemFee(await signers[1].getAddress()),
                "redeem fee for 20 more claims"
            )
                .to.emit(feeManagerFacet, "FeeRedeemed")
                .withArgs(await signers[1].getAddress(), 20);
            // check balances
            expect(
                await token.balanceOf(bank.address),
                "bank now has totally 30*initialFeePerClaim less tokens"
            ).to.equal(amount - 30 * initialFeePerClaim);
            expect(
                await bank.balanceOf(feeManagerFacet.address),
                "feeManager's bank balance got decreased by 30*initialFeePerClaim"
            ).to.equal(amount - 30 * initialFeePerClaim);
            expect(
                await token.balanceOf(await signers[1].getAddress()),
                "validator now has totally 30*initialFeePerClaim tokens"
            ).to.equal(30 * initialFeePerClaim);
        } else {
            // test delegate
            var claim = "0x" + "1".repeat(64);
            // let signers[1] make 10 claims
            let num_claims = 10;
            for (let i = 0; i < num_claims; i++) {
                await passInputAccumulationPeriod();
                await rollupsFacet.connect(signers[1]).claim(claim);
                await passChallengePeriod();
                await rollupsFacet.finalizeEpoch();
            }

            let state = JSON.parse(await getState(initialState));
            // since fee manager doesn't have any deposit yet
            // now its balance is 0 and uncommitted balance is negative
            expect(state.bank_balance, "balance is 0").to.equal("0x0");
            expect(
                state.uncommitted_balance,
                "uncommitted balance is negative"
            ).to.equal(num_claims * initialFeePerClaim * -1);
            // other fields are the same as initial
            same_as_initial_value("fee_per_claim", state);
            same_as_initial_value("validator_redeemed", state);

            // we now make a deposit to cover the negative uncommitted balance
            let amount = state.uncommitted_balance * -1;
            await fundFeeManager(amount);
            // update state
            state = JSON.parse(await getState(initialState));
            expect(
                parseInt(state.bank_balance, 16),
                "balance should be 100"
            ).to.equal(100);
            expect(
                state.uncommitted_balance,
                "uncommitted balance is 0"
            ).to.equal(0);

            // let signers[1] redeem its fees now
            await feeManagerFacet
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress());

            // update state
            state = JSON.parse(await getState(initialState));

            // signers[1] should be in `validator_redeemed`
            expect(
                state.validator_redeemed[0].validator_address,
                "signers[1] should be in `validator_redeemed`"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(
                parseInt(state.validator_redeemed[0].num_claims_redeemed, 16),
                "#claims redeemed should be 10"
            ).to.equal(num_claims);
            for (let i = 1; i < 8; i++) {
                expect(
                    state.validator_redeemed[i],
                    "the rest in `validator_redeemed` should be null"
                ).to.equal(null);
            }

            // uncommitted_balance is still 0
            expect(
                state.uncommitted_balance,
                "uncommitted_balance is still 0"
            ).to.equal(0);
            // after redemption, bank_balance becomes 0
            expect(
                state.bank_balance,
                "after redemption, bank_balance becomes 0"
            ).to.equal("0x0");

            // make another deposit to fee manager
            await fundFeeManager(10000);
            // update state
            state = JSON.parse(await getState(initialState));
            expect(
                parseInt(state.bank_balance, 16),
                "the bank_balance should be 10k"
            ).to.equal(10000);
            expect(
                state.uncommitted_balance,
                "uncommitted_balance should also be 10k"
            ).to.equal(10000);
        }
    });

    it("redeemFee on other's behalf", async () => {
        if (!enableDelegate) {
            //owner fund FeeManager
            let amount = 10000;
            await fundFeeManager(amount);

            // assume signers[1] makes 10 claims
            await debugFacet._setNumClaims(1, 10);

            // let signers[0] help signers[1] redeemFee
            await expect(
                feeManagerFacet.redeemFee(await signers[1].getAddress()),
                "signers[0] helps signers[1] redeemFee"
            )
                .to.emit(feeManagerFacet, "FeeRedeemed")
                .withArgs(await signers[1].getAddress(), 10);

            // check balances
            expect(
                await token.balanceOf(bank.address),
                "signers[0] helped signers[1]: bank now has 10*initialFeePerClaim less tokens"
            ).to.equal(amount - 10 * initialFeePerClaim);
            expect(
                await bank.balanceOf(feeManagerFacet.address),
                "signers[0] helped signers[1]: feeManager's bank balance decreased by 10*initialFeePerClaim"
            ).to.equal(amount - 10 * initialFeePerClaim);
            expect(
                await token.balanceOf(await signers[1].getAddress()),
                "signers[0] helped signers[1]: signers[1] now has 10*initialFeePerClaim tokens"
            ).to.equal(10 * initialFeePerClaim);
            expect(
                await token.balanceOf(await signers[0].getAddress()),
                "signers[0] helped signers[1]: signers[0] balance doesn't change"
            ).to.equal(tokenSupply - amount);
        } else {
            // test delegate
            var claim = "0x" + "1".repeat(64);
            // let signers[1] make 10 claims
            let num_claims = 10;
            for (let i = 0; i < num_claims; i++) {
                await passInputAccumulationPeriod();
                await rollupsFacet.connect(signers[1]).claim(claim);
                await passChallengePeriod();
                await rollupsFacet.finalizeEpoch();
            }
            //deposit 10k to fee manager
            await fundFeeManager(10000);
            // let signers[0] help signers[1] redeem fee
            await feeManagerFacet.redeemFee(await signers[1].getAddress());

            let state = JSON.parse(await getState(initialState));
            // signers[1] should be in `validator_redeemed`
            expect(
                state.validator_redeemed[0].validator_address,
                "signers[1] should be in `validator_redeemed`"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(
                parseInt(state.validator_redeemed[0].num_claims_redeemed, 16),
                "#claims redeemed should be 10"
            ).to.equal(num_claims);
            for (let i = 1; i < 8; i++) {
                expect(
                    state.validator_redeemed[i],
                    "the rest in `validator_redeemed` should be null"
                ).to.equal(null);
            }
            // after redemption, bank_balance becomes 10k - redeemedValue
            expect(
                parseInt(state.bank_balance, 16),
                "after redemption, bank_balance becomes 10k - redeemedValue"
            ).to.equal(10000 - num_claims * initialFeePerClaim);
            // uncommitted_balance should be the same as bank_balance
            expect(
                state.uncommitted_balance,
                "uncommitted_balance should be the same as bank_balance"
            ).to.equal(10000 - num_claims * initialFeePerClaim);
        }
    });

    // only owner can call resetFeePerClaim
    it("only owner can call resetFeePerClaim", async () => {
        await expect(
            feeManagerFacet.connect(signers[1]).resetFeePerClaim(30),
            "only owner"
        ).to.be.reverted;
    });

    // reset fee per claim
    it("reset feePerClaim", async () => {
        if (!enableDelegate) {
            //owner fund FeeManager
            let amount = 10000;
            await fundFeeManager(amount);

            // assume signers[1], signers[2], signers[3] are the validator set
            await debugFacet._setNumClaims(1, 10);
            await debugFacet._setNumClaims(2, 20);
            await debugFacet._setNumClaims(3, 0);

            // assume the signers[1] has already claimed
            await feeManagerFacet
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress());

            // get validators' balances before resetting fees
            let token_balance1_before = await token.balanceOf(
                await signers[1].getAddress()
            );
            let token_balance2_before = await token.balanceOf(
                await signers[2].getAddress()
            );
            let token_balance3_before = await token.balanceOf(
                await signers[3].getAddress()
            );
            let token_balance_before = await token.balanceOf(bank.address);
            let bank_balance_before = await bank.balanceOf(
                feeManagerFacet.address
            );

            let newFeePerClaim = 30;
            // reset fee from 10 -> 30
            expect(await feeManagerFacet.resetFeePerClaim(newFeePerClaim))
                .to.emit(feeManagerFacet, "FeePerClaimReset")
                .withArgs(newFeePerClaim);

            // get new balances
            let token_balance1_after = await token.balanceOf(
                await signers[1].getAddress()
            );
            let token_balance2_after = await token.balanceOf(
                await signers[2].getAddress()
            );
            let token_balance3_after = await token.balanceOf(
                await signers[3].getAddress()
            );
            let token_balance_after = await token.balanceOf(bank.address);
            let bank_balance_after = await bank.balanceOf(
                feeManagerFacet.address
            );

            // check new balances
            expect(
                token_balance1_after,
                "balance of signers[1] stays the same"
            ).to.equal(token_balance1_before);
            expect(
                token_balance2_after.toNumber(),
                "signers[2] gets fees for 20 claims"
            ).to.equal(
                token_balance2_before.toNumber() + 20 * initialFeePerClaim
            );
            expect(
                token_balance3_after,
                "balance of signers[3] stays the same"
            ).to.equal(token_balance3_before);
            expect(
                token_balance_after,
                "bank pays fees for 20 claims"
            ).to.equal(
                token_balance_before.toNumber() - 20 * initialFeePerClaim
            );
            expect(
                bank_balance_after,
                "feeManager's bank balance is decreased by 20*initialFeePerClaim"
            ).to.equal(
                bank_balance_before.toNumber() - 20 * initialFeePerClaim
            );

            // feePerClaim is updated
            expect(
                await debugFacet._getFeePerClaim(),
                "check updated feeManager"
            ).to.equal(newFeePerClaim);

            // now onwards, validators can claim based on new rates
            // assume signers[3] makes 10 claims now and claims fees on its own
            await debugFacet._setNumClaims(3, 10);
            await feeManagerFacet
                .connect(signers[3])
                .redeemFee(await signers[3].getAddress());
            expect(
                await token.balanceOf(await signers[3].getAddress()),
                "balance of signers[3] after resetting fees and making claims"
            ).to.equal(10 * newFeePerClaim);
            expect(
                await token.balanceOf(bank.address),
                "bank pays fees for 10 claims"
            ).to.equal(token_balance_after.toNumber() - 10 * newFeePerClaim);
            expect(
                await bank.balanceOf(feeManagerFacet.address),
                "feeManager's bank balance is decreased by 10*newFeePerClaim"
            ).to.equal(bank_balance_after.toNumber() - 10 * newFeePerClaim);
        } else {
            // test delegate
            var claim = "0x" + "1".repeat(64);
            // let signers[1] make 10 claims
            let num_claims = 10;
            for (let i = 0; i < num_claims; i++) {
                await passInputAccumulationPeriod();
                await rollupsFacet.connect(signers[1]).claim(claim);
                await passChallengePeriod();
                await rollupsFacet.finalizeEpoch();
            }
            // deposit 10k to fee manager
            await fundFeeManager(10000);
            // instead of signers[1] redeeming fee, the fee_per_claim is reset
            // reset fee from 10 -> 30
            let newFeePerClaim = 30;
            await feeManagerFacet.resetFeePerClaim(newFeePerClaim);

            let state = JSON.parse(await getState(initialState));
            // now the fee_per_claim should be `newFeePerClaim`
            expect(
                parseInt(state.fee_per_claim, 16),
                "now the fee_per_claim should be `newFeePerClaim`"
            ).to.equal(newFeePerClaim);
            // signers[1] should automatically be redeemed fees
            expect(
                state.validator_redeemed[0].validator_address,
                "signers[1] should automatically be redeemed fees"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(
                parseInt(state.validator_redeemed[0].num_claims_redeemed, 16),
                "#claims redeemed should be 10"
            ).to.equal(num_claims);
            for (let i = 1; i < 8; i++) {
                expect(
                    state.validator_redeemed[i],
                    "the rest in `validator_redeemed` should be null"
                ).to.equal(null);
            }
            // after automatic redemption, bank_balance should less some balance,
            // based on the previous fee_per_claim
            expect(
                parseInt(state.bank_balance, 16),
                "after automatic redemption, bank_balance becomes 10k - redeemedValue"
            ).to.equal(10000 - num_claims * initialFeePerClaim);
            // uncommitted_balance should be the same as bank_balance
            expect(
                state.uncommitted_balance,
                "uncommitted_balance should be the same as bank_balance"
            ).to.equal(10000 - num_claims * initialFeePerClaim);

            let current_bank_balance = state.bank_balance;
            let current_uncommitted_balance = state.uncommitted_balance;
            // signers[1] makes another claim
            await passInputAccumulationPeriod();
            await rollupsFacet.connect(signers[1]).claim(claim);
            await passChallengePeriod();
            await rollupsFacet.finalizeEpoch();
            // update state
            state = JSON.parse(await getState(initialState));
            // bank_balance should stay the same
            expect(
                state.bank_balance,
                "bank_balance should stay the same"
            ).to.equal(current_bank_balance);
            // uncommitted_balance should less the new fee_per_claim value
            expect(
                state.uncommitted_balance,
                "uncommitted_balance should less the new fee_per_claim value"
            ).to.equal(current_uncommitted_balance - newFeePerClaim);
        }
    });

    // when a validator gets removed
    it("test when a validator gets removed", async () => {
        if (!enableDelegate) {
            //owner fund FeeManager
            let amount = 10000;
            await fundFeeManager(amount);

            // assume signers[1] makes 10 claims and redeem them
            await debugFacet._setNumClaims(1, 10);
            await feeManagerFacet
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress());

            // assume signers[1] makes another 10 claims
            await debugFacet._setNumClaims(1, 20);
            // but then lost a dispute
            // its number of claims will be set to 0 by validator manager
            await debugFacet._setNumClaims(1, 0);
            // so signers[1] will not be able to redeem any more fees
            await expect(
                feeManagerFacet
                    .connect(signers[1])
                    .redeemFee(await signers[1].getAddress()),
                "signers[1] will not be able to redeem anymore"
            ).to.be.reverted;
        } else {
            // test delegate
            var claim = "0x" + "1".repeat(64);
            var claim2 = "0x" + "2".repeat(64);
            // deposit 10k to fee manager
            await fundFeeManager(10000);

            // let all 8 validators make a claim and finalize
            await passInputAccumulationPeriod();
            for (let i = 0; i < 8; i++) {
                await rollupsFacet.connect(signers[i]).claim(claim);
            }
            // currently every validator has 1 claim that's redeemable
            // assume next epoch, signers[0] wins a dispute over signers[1]
            await passInputAccumulationPeriod();
            await rollupsFacet.connect(signers[0]).claim(claim);
            await rollupsFacet.connect(signers[1]).claim(claim2);
            await passChallengePeriod();
            await rollupsFacet.finalizeEpoch();

            // there will be totally 8 claims that can be redeemed
            // signers[0] has 2 and signers[1] has none
            let state = JSON.parse(await getState(initialState));
            // uncommitted_balance will less 8 claims
            expect(
                state.uncommitted_balance,
                "uncommitted_balance will less 8 claims"
            ).to.equal(10000 - 8 * initialFeePerClaim);
            // bank_balance should be 10k
            expect(
                parseInt(state.bank_balance, 16),
                "fee manager has 10k tokens deposited"
            ).to.equal(10000);
            // the rest are the same as initial
            same_as_initial_value("fee_per_claim", state);
            same_as_initial_value("validator_redeemed", state);

            // let all validators redeem fees
            for (let i = 0; i < 8; i++) {
                if (i == 1) {
                    // signers[1] cannot redeem, will revert
                    await expect(
                        feeManagerFacet
                            .connect(signers[1])
                            .redeemFee(await signers[1].getAddress())
                    ).to.be.reverted;
                    continue;
                }
                await feeManagerFacet
                    .connect(signers[i])
                    .redeemFee(await signers[i].getAddress());
            }

            // update state
            state = JSON.parse(await getState(initialState));
            // for signers[0]
            expect(
                state.validator_redeemed[0].validator_address,
                "the first one in `validator_redeemed` is signers[0]"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                parseInt(state.validator_redeemed[0].num_claims_redeemed, 16),
                "signers[0] redeems 2 claims"
            ).to.equal(2);
            // for signers[2-7]
            for (let i = 1; i < 7; i++) {
                expect(
                    state.validator_redeemed[i].validator_address,
                    "signers[1] will not be in `validator_redeemed`, so i+1 to skip signers[1]"
                ).to.equal((await signers[i + 1].getAddress()).toLowerCase());
                expect(
                    parseInt(
                        state.validator_redeemed[i].num_claims_redeemed,
                        16
                    ),
                    "#claims should be 1, for validators except signers[0] and signers[1]"
                ).to.equal(1);
            }
            // signers[1] is not in `validator_redeemed`
            expect(
                state.validator_redeemed[7],
                "signers[1] is not in `validator_redeemed`"
            ).to.equal(null);

            // uncommitted_balance is the same as before redemption
            expect(
                state.uncommitted_balance,
                "uncommitted_balance is the same as before redemption"
            ).to.equal(10000 - 8 * initialFeePerClaim);
            // bank_balance now should be the same as uncommitted_balance
            expect(
                parseInt(state.bank_balance, 16),
                "bank_balance now should be the same as uncommitted_balance"
            ).to.equal(10000 - 8 * initialFeePerClaim);
        }
    });

    it("test whether we can withdraw tokens from Bank via the ERC-20 Portal", async () => {
        // fund 10000 tokens
        let amount = 10000;
        await fundFeeManager(amount);

        // create fake input
        const erc20 = bank.address;
        const sender = await signers[0].getAddress();

        let data = ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [erc20, sender, amount]
        );

        // check erc20Withdrawal reverts
        await expect(debugFacet._erc20Withdrawal(data)).to.be.revertedWith(
            "function selector was not recognized and there's no fallback function"
        );
    });
});

// helper function
function same_as_initial_value(s: string, state: any) {
    switch (s) {
        case "fee_per_claim": {
            expect(state.fee_per_claim, "fee_per_claim should be 16").to.equal(
                "0xa"
            );
            break;
        }
        case "validator_redeemed": {
            expect(
                state.validator_redeemed.length,
                "should have 8 Options"
            ).to.equal(8);
            for (let i = 0; i < 8; i++) {
                expect(
                    state.validator_redeemed[i],
                    "each Option should be null"
                ).to.equal(null);
            }
            break;
        }
        case "bank_balance": {
            expect(
                state.bank_balance,
                "fee manager should have 0 balance"
            ).to.equal("0x0");
            break;
        }
        case "uncommitted_balance": {
            expect(
                state.uncommitted_balance,
                "fee manager should have 0 uncommitted balance"
            ).to.equal(0);
            break;
        }
    }
}
