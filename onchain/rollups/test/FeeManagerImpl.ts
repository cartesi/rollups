import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import { FeeManagerImpl } from "../src/types/FeeManagerImpl";
import { FeeManagerImpl__factory } from "../src/types/factories/FeeManagerImpl__factory";
import { SimpleToken } from "../src/types/SimpleToken";
import { SimpleToken__factory } from "../src/types/factories/SimpleToken__factory";
import { getState } from "./utils";

use(solidity);

describe("FeeManager Implementation", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    let signers: Signer[];
    let mockValidatorManager: MockContract;
    let token: SimpleToken;
    let feeManager: FeeManagerImpl;
    let tokenSupply = 1000000; // assume FeeManagerImpl contract owner has 1 million tokens (ignore decimals)
    let initialFeePerClaim = 10; // set initial fees per claim as 10 token

    beforeEach(async () => {
        // get signers
        signers = await ethers.getSigners();

        await deployments.fixture(["RollupsImpl"]);

        // mock ValidatorManagerCCI
        const ValidatorManagerCCI = await deployments.getArtifact(
            "ValidatorManagerClaimsCountedImpl"
        );
        mockValidatorManager = await deployMockContract(
            signers[0],
            ValidatorManagerCCI.abi
        );

        // deploy ERC20 token
        let deployedToken = await deployments.get("SimpleToken");
        token = SimpleToken__factory.connect(deployedToken.address, signers[0]);

        // deploy LibClaimsMask
        let libClaimsMask = await deployments.get("LibClaimsMask");
        const libClaimsMaskAddress = libClaimsMask.address;

        // deploy Fee Manager
        let deployedFeeManager = await deployments.deploy("FeeManagerImpl", {
            from: await signers[0].getAddress(),
            libraries: {
                LibClaimsMask: libClaimsMaskAddress,
            },
            args: [
                mockValidatorManager.address,
                token.address,
                initialFeePerClaim,
            ],
        });
        // let signers[0] be the owner of feeManager
        feeManager = FeeManagerImpl__factory.connect(
            deployedFeeManager.address,
            signers[0]
        );
    });

    it("test initial feePerClaim", async () => {
        expect(await feeManager.feePerClaim(), "initial feePerClaim").to.equal(
            initialFeePerClaim
        );

        // test delegate
        if (enableDelegate) {
            let initialState = JSON.stringify({
                fee_manager_address: feeManager.address,
            });

            let state = JSON.parse(await getState(initialState));

            console.log(state);
        }
    });

    it("test constructor event FeeManagerCreated", async () => {
        let eventFilter = feeManager.filters.FeeManagerCreated(
            null,
            null,
            null
        );
        let event = await feeManager.queryFilter(eventFilter);
        let eventArgs = event[0]["args"];

        expect(
            eventArgs["_ValidatorManagerCCI"],
            "MockValidatorManagerCCI address"
        ).to.equal(mockValidatorManager.address);
        expect(eventArgs["_ERC20"], "ERC20 token address").to.equal(
            token.address
        );
        expect(eventArgs["_feePerClaim"], "feePerClaim").to.equal(
            initialFeePerClaim
        );
    });

    it("fund the FeeManager contract and emit event", async () => {
        expect(
            await token.balanceOf(feeManager.address),
            "initially the contract has no erc20 tokens"
        ).to.equal(0);

        // fund 10000 tokens
        let amount = 10000;
        expect(await token.transfer(feeManager.address, amount))
            .to.emit(token, "Transfer")
            .withArgs(
                await signers[0].getAddress(),
                feeManager.address,
                amount
            );

        expect(
            await token.balanceOf(feeManager.address),
            "feeManager now has 10k erc20 tokens"
        ).to.equal(amount);
        expect(
            await token.balanceOf(await signers[0].getAddress()),
            "owner has 10k less tokens"
        ).to.equal(tokenSupply - amount);

        // again, fund 10000 tokens
        expect(await token.transfer(feeManager.address, amount))
            .to.emit(token, "Transfer")
            .withArgs(
                await signers[0].getAddress(),
                feeManager.address,
                amount
            );

        expect(
            await token.balanceOf(feeManager.address),
            "feeManager now has 20k erc20 tokens"
        ).to.equal(amount * 2);
        expect(
            await token.balanceOf(await signers[0].getAddress()),
            "owner has 20k less tokens"
        ).to.equal(tokenSupply - amount * 2);
    });

    // test numClaimsRedeemable
    it("test numClaimsRedeemable()", async () => {
        // revert on address 0
        let address_zero = "0x0000000000000000000000000000000000000000";
        await expect(
            feeManager.numClaimsRedeemable(address_zero),
            "should revert on address 0"
        ).to.be.revertedWith("address should not be 0");

        // assume signers[1] is a validator with 0 redeemable claims
        await mockValidatorManager.mock.getValidatorIndex.returns(0);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(0);
        expect(
            await feeManager.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "initially, no redeemable claims"
        ).to.equal(0);

        // assume signers[1] makes 10 claims
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(10);
        expect(
            await feeManager.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "now there are 10 claims redeemable"
        ).to.equal(10);

        // owner funds the FeeManager and signers[1] redeem fees
        let amount = 10000;
        await token.transfer(feeManager.address, amount);
        await feeManager.redeemFee(await signers[1].getAddress());

        // after having redeemed, no more redeemable claims
        expect(
            await feeManager.callStatic.numClaimsRedeemable(
                await signers[1].getAddress()
            ),
            "redeemable claims are all redeemed"
        ).to.equal(0);

        // test default "underflow check" by sol0.8
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(0);
        await expect(
            feeManager.numClaimsRedeemable(await signers[1].getAddress()),
            "0 - 10"
        ).to.be.reverted;
    });

    // test getNumClaimsRedeemed
    it("test getNumClaimsRedeemed()", async () => {
        // revert on address 0
        let address_zero = "0x0000000000000000000000000000000000000000";
        await expect(
            feeManager.getNumClaimsRedeemed(address_zero),
            "getNumClaimsRedeemed() should revert on address 0"
        ).to.be.revertedWith("address should not be 0");

        // assume signers[1] is a validator with 0 redeemed claims
        await mockValidatorManager.mock.getValidatorIndex.returns(0);
        expect(
            await feeManager.callStatic.getNumClaimsRedeemed(
                await signers[1].getAddress()
            ),
            "initially, no redeemed claims"
        ).to.equal(0);

        // assume signers[1] redeems 10 claims
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(10);
        let amount = 10000;
        await token.transfer(feeManager.address, amount);
        await feeManager.redeemFee(await signers[1].getAddress());

        expect(
            await feeManager.callStatic.getNumClaimsRedeemed(
                await signers[1].getAddress()
            ),
            "now #redeemed should be 10"
        ).to.equal(10);
    });

    // redeem fees
    it("test redeemFee() when no claims have been made", async () => {
        // assume signers[1] is a validator
        await mockValidatorManager.mock.getValidatorIndex.returns(0);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(0);

        await expect(
            feeManager.redeemFee(await signers[1].getAddress()),
            "no claims made"
        ).to.be.revertedWith("nothing to redeem yet");
    });

    it("redeemFee on his/her own", async () => {
        //owner fund FeeManager
        let amount = 10000;
        await token.transfer(feeManager.address, amount);

        // assume signers[1] is a validator
        await mockValidatorManager.mock.getValidatorIndex.returns(0);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(10);

        await expect(
            feeManager
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress()),
            "redeem fee for 10 claims"
        )
            .to.emit(feeManager, "FeeRedeemed")
            .withArgs(await signers[1].getAddress(), 10 * initialFeePerClaim);

        // check balances
        expect(
            await token.balanceOf(feeManager.address),
            "feeManager now has 10*initialFeePerClaim less tokens"
        ).to.equal(amount - 10 * initialFeePerClaim);
        expect(
            await token.balanceOf(await signers[1].getAddress()),
            "validator now has 10*initialFeePerClaim tokens"
        ).to.equal(10 * initialFeePerClaim);

        // redeemFee again
        await expect(
            feeManager
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress()),
            "no additional claims made"
        ).to.be.revertedWith("nothing to redeem yet");

        // make more claims and then redeemFee
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(30);
        await expect(
            feeManager
                .connect(signers[1])
                .redeemFee(await signers[1].getAddress()),
            "redeem fee for 20 more claims"
        )
            .to.emit(feeManager, "FeeRedeemed")
            .withArgs(await signers[1].getAddress(), 20 * initialFeePerClaim);
        // check balances
        expect(
            await token.balanceOf(feeManager.address),
            "feeManager now has totally 30*initialFeePerClaim less tokens"
        ).to.equal(amount - 30 * initialFeePerClaim);
        expect(
            await token.balanceOf(await signers[1].getAddress()),
            "validator now has totally 30*initialFeePerClaim tokens"
        ).to.equal(30 * initialFeePerClaim);
    });

    it("redeemFee on other's behalf", async () => {
        //owner fund FeeManager
        let amount = 10000;
        await token.transfer(feeManager.address, amount);

        // assume signers[1] is a validator
        await mockValidatorManager.mock.getValidatorIndex.returns(0);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex.returns(10);

        // let signers[0] help signers[1] redeemFee
        await expect(
            feeManager.redeemFee(await signers[1].getAddress()),
            "signers[0] helps signers[1] redeemFee"
        )
            .to.emit(feeManager, "FeeRedeemed")
            .withArgs(await signers[1].getAddress(), 10 * initialFeePerClaim);

        // check balances
        expect(
            await token.balanceOf(feeManager.address),
            "signers[0] helped signers[1]: feeManager now has 10*initialFeePerClaim less tokens"
        ).to.equal(amount - 10 * initialFeePerClaim);
        expect(
            await token.balanceOf(await signers[1].getAddress()),
            "signers[0] helped signers[1]: signers[1] now has 10*initialFeePerClaim tokens"
        ).to.equal(10 * initialFeePerClaim);
        expect(
            await token.balanceOf(await signers[0].getAddress()),
            "signers[0] helped signers[1]: signers[0] balance doesn't change"
        ).to.equal(tokenSupply - amount);
    });

    // only owner can call resetFeePerClaim
    it("only owner can call resetFeePerClaim", async () => {
        await expect(
            feeManager.connect(signers[1]).resetFeePerClaim(30),
            "only owner"
        ).to.be.reverted;
    });

    // reset fee per claim
    it("reset feePerClaim", async () => {
        //owner fund FeeManager
        let amount = 10000;
        await token.transfer(feeManager.address, amount);

        // assume signers[1], signers[2], signers[3] are the validator set
        await mockValidatorManager.mock.maxNumValidators.returns(3);
        await mockValidatorManager.mock.validators
            .withArgs(0)
            .returns(await signers[1].getAddress());
        await mockValidatorManager.mock.validators
            .withArgs(1)
            .returns(await signers[2].getAddress());
        await mockValidatorManager.mock.validators
            .withArgs(2)
            .returns(await signers[3].getAddress());
        await mockValidatorManager.mock.getValidatorIndex
            .withArgs(await signers[1].getAddress())
            .returns(0);
        await mockValidatorManager.mock.getValidatorIndex
            .withArgs(await signers[2].getAddress())
            .returns(1);
        await mockValidatorManager.mock.getValidatorIndex
            .withArgs(await signers[3].getAddress())
            .returns(2);

        // the number of claims they have made
        await mockValidatorManager.mock.getNumberOfClaimsByIndex
            .withArgs(0)
            .returns(10);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex
            .withArgs(1)
            .returns(20);
        await mockValidatorManager.mock.getNumberOfClaimsByIndex
            .withArgs(2)
            .returns(0);

        // assume the signers[1] has already claimed
        await feeManager
            .connect(signers[1])
            .redeemFee(await signers[1].getAddress());

        // get validators' balances before resetting fees
        let balance0_before = await token.balanceOf(
            await signers[1].getAddress()
        );
        let balance1_before = await token.balanceOf(
            await signers[2].getAddress()
        );
        let balance2_before = await token.balanceOf(
            await signers[3].getAddress()
        );

        let newFeePerClaim = 30;
        // reset fee from 10 -> 30
        expect(await feeManager.resetFeePerClaim(newFeePerClaim))
            .to.emit(feeManager, "FeePerClaimReset")
            .withArgs(newFeePerClaim);

        // get new balances
        let balance0_after = await token.balanceOf(
            await signers[1].getAddress()
        );
        let balance1_after = await token.balanceOf(
            await signers[2].getAddress()
        );
        let balance2_after = await token.balanceOf(
            await signers[3].getAddress()
        );

        // check new balances
        expect(balance0_after, "balance of signers[1] stays the same").to.equal(
            balance0_before
        );
        expect(
            balance1_after.toNumber(),
            "signers[2] gets fees for 20 claims"
        ).to.equal(balance1_before.toNumber() + 20 * initialFeePerClaim);
        expect(balance2_after, "balance of signers[3] stays the same").to.equal(
            balance2_before
        );

        // feePerClaim is updated
        expect(
            await feeManager.feePerClaim(),
            "check updated feeManager"
        ).to.equal(newFeePerClaim);

        // now onwards, validators can claim based on new rates
        // assume signers[3] makes 10 claims now and claims fees on its own
        await mockValidatorManager.mock.getNumberOfClaimsByIndex
            .withArgs(2)
            .returns(10);
        await feeManager
            .connect(signers[3])
            .redeemFee(await signers[3].getAddress());
        expect(
            await token.balanceOf(await signers[3].getAddress()),
            "balance of signers[3] after resetting fees and making claims"
        ).to.equal(10 * newFeePerClaim);
    });
});
