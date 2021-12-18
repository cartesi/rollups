import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { TestClaimsMaskLibrary } from "../dist/src/types/TestClaimsMaskLibrary";
import { TestClaimsMaskLibrary__factory } from "../dist/src/types/factories/TestClaimsMaskLibrary__factory";
import { BigNumber } from "ethers";

use(solidity);

describe("Test ClaimsMaskLibrary", () => {
    let signers: Signer[];
    let claimsMaskLibrary: TestClaimsMaskLibrary;

    beforeEach(async () => {
        await deployments.fixture(["RollupsImpl"]);

        // get signers
        signers = await ethers.getSigners();

        // deploy ClaimsMaskLibrary
        const deployedClaimsMaskLibrary = await deployments.deploy(
            "ClaimsMaskLibrary",
            {
                from: await signers[0].getAddress(),
            }
        );
        const claimsMaskLibraryAddress = deployedClaimsMaskLibrary.address;

        // deploy TestClaimsMaskLibrary
        const { address } = await deployments.deploy("TestClaimsMaskLibrary", {
            from: await signers[0].getAddress(),
            libraries: {
                ClaimsMaskLibrary: claimsMaskLibraryAddress,
            },
        });
        claimsMaskLibrary = TestClaimsMaskLibrary__factory.connect(
            address,
            signers[0]
        );
    });

    it("create a ClaimsMask", async () => {
        // case 1: 0
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);
        expect(claimsMask, "new claimsMask").to.equal(0);

        // case 2: random number
        let randomNumber = BigNumber.from(ethers.utils.randomBytes(32));
        claimsMask = await claimsMaskLibrary.newClaimsMask(randomNumber);
        expect(claimsMask, "random claimsMask").to.equal(randomNumber);
    });

    it("create a ClaimsMask with consensus goal set", async () => {
        // revert when more than 8 validators
        let numValidators = 9;
        await expect(
            claimsMaskLibrary.newClaimsMaskWithConsensusGoalSet(numValidators),
            "fail to create ClaimsMask"
        ).to.be.revertedWith("up to 8 validators");

        // loop scenarios from 1 to 8 validators
        for (numValidators = 1; numValidators <= 8; numValidators++) {
            expect(
                BigNumber.from(
                    await claimsMaskLibrary.newClaimsMaskWithConsensusGoalSet(
                        numValidators
                    )
                ),
                "create ClaimsMask with 1~8 validators"
            ).to.equal(
                BigNumber.from((1 << numValidators) - 1).mul(
                    BigNumber.from(2).pow(240)
                )
            ); //((1<<numValidators)-1) << 240
        }
    });

    it("get #claims from ClaimsMask", async () => {
        // revert when index is greater than 7
        let validatorIndex = 8;
        await expect(
            claimsMaskLibrary.getNumClaims(0, validatorIndex),
            "fail to get #claims"
        ).to.be.revertedWith("index out of range");

        // all #claims are 0 by default
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);
        for (validatorIndex = 0; validatorIndex < 8; validatorIndex++) {
            expect(
                await claimsMaskLibrary.getNumClaims(
                    claimsMask,
                    validatorIndex
                ),
                "check default #claims for all validators"
            ).to.equal(0);
        }

        // let the #claims be 0 for validator_0, 1 for validator_1, ..., 7 for validator_7
        claimsMask = BigNumber.from(0);
        for (let i = 0; i < 8; i++) {
            claimsMask = claimsMask.add(
                BigNumber.from(i).mul(BigNumber.from(2).pow(30 * i))
            ); // i << (30*i)
        }
        for (validatorIndex = 0; validatorIndex < 8; validatorIndex++) {
            expect(
                await claimsMaskLibrary.getNumClaims(
                    claimsMask,
                    validatorIndex
                ),
                "check #claims for all validators"
            ).to.equal(validatorIndex);
        }
    });

    it("increase #claims", async () => {
        // revert when index is greater than 7
        let validatorIndex = 8;
        await expect(
            claimsMaskLibrary.increaseNumClaims(0, validatorIndex, 0),
            "fail to increase #claims"
        ).to.be.revertedWith("index out of range");

        // increase #claims by 1 for validator_1
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);
        claimsMask = await claimsMaskLibrary.increaseNumClaims(
            claimsMask,
            1,
            1
        );
        // check validator_1
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 1),
            "validator_1 should have #claims as 1"
        ).to.equal(1);
        // check other validators
        for (validatorIndex = 0; validatorIndex < 8; validatorIndex++) {
            if (validatorIndex == 1) continue;
            expect(
                await claimsMaskLibrary.getNumClaims(
                    claimsMask,
                    validatorIndex
                ),
                "other validators should have #claims as 0"
            ).to.equal(0);
        }

        // increase #claims by 10 for validator_4
        claimsMask = await claimsMaskLibrary.increaseNumClaims(
            claimsMask,
            4,
            10
        );
        // check validator_4
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 4),
            "validator_4 should have #claims as 10"
        ).to.equal(10);
        // check other validators
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 0),
            "validator_0 should still have #claims as 0"
        ).to.equal(0);
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 1),
            "validator_1 should still have #claims as 1"
        ).to.equal(1);
        for (validatorIndex = 2; validatorIndex < 8; validatorIndex++) {
            if (validatorIndex == 4) continue;
            expect(
                await claimsMaskLibrary.getNumClaims(
                    claimsMask,
                    validatorIndex
                ),
                "other validators should still have #claims as 0"
            ).to.equal(0);
        }

        // revert if the increase is too big
        let valueTooBig = BigNumber.from(2).pow(30); // 1<<30
        await expect(
            claimsMaskLibrary.increaseNumClaims(claimsMask, 0, valueTooBig),
            "increase value too big"
        ).to.be.revertedWith("ClaimsMask Overflow");

        // works fine increasing to (valueTooBig - 1)
        claimsMask = await claimsMaskLibrary.increaseNumClaims(
            claimsMask,
            0,
            valueTooBig.sub(1)
        );
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 0),
            "works fine increasing to (valueTooBig - 1)"
        ).to.equal(valueTooBig.sub(1));
    });

    it("set #claims", async () => {
        // revert when index is greater than 7
        let validatorIndex = 8;
        await expect(
            claimsMaskLibrary.setNumClaims(0, validatorIndex, 0),
            "fail to set #claims"
        ).to.be.revertedWith("index out of range");

        // revert when #claims too big
        let valueTooBig = BigNumber.from(2).pow(30); // 1<<30
        await expect(
            claimsMaskLibrary.setNumClaims(0, 0, valueTooBig),
            "set value overflow"
        ).to.be.revertedWith("ClaimsMask Overflow");

        // works fine with (valueTooBig - 1)
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);
        claimsMask = await claimsMaskLibrary.setNumClaims(
            claimsMask,
            0,
            valueTooBig.sub(1)
        );
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 0),
            "works fine with (valueTooBig - 1)"
        ).to.equal(valueTooBig.sub(1));

        // set #claims back to 0 for validator_0
        claimsMask = await claimsMaskLibrary.setNumClaims(claimsMask, 0, 0);
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 0),
            "value set back to 0"
        ).to.equal(0);

        // set #claims to 4 for validator_4
        claimsMask = await claimsMaskLibrary.setNumClaims(claimsMask, 4, 4);
        expect(
            await claimsMaskLibrary.getNumClaims(claimsMask, 4),
            "#claims is 4 for validator_4"
        ).to.equal(4);
    });

    it("get agreement mask", async () => {
        // let agreement mask initially be 11111111
        let claimsMask = ethers.constants.MaxUint256;
        // initial agreement mask should be all 1, each time right shift 1
        for (let i = 8; i > 0; i--) {
            expect(
                await claimsMaskLibrary.getAgreementMask(claimsMask),
                "initial agreement mask should be all 1, each time right shift 1"
            ).to.equal((1 << i) - 1);
            claimsMask = claimsMask.div(2); // claimsMask >> 1
        }

        // let agreement mask be 00000100
        claimsMask = BigNumber.from(2).pow(256 - 6); // 1 << 250
        expect(
            await claimsMaskLibrary.getAgreementMask(claimsMask),
            "agreemnt mask should be 00000100"
        ).to.equal(4);
    });

    it("clear agreement mask", async () => {
        // let agreement mask be 11111111
        let claimsMask = ethers.constants.MaxUint256;
        claimsMask = await claimsMaskLibrary.clearAgreementMask(claimsMask);
        expect(
            await claimsMaskLibrary.getAgreementMask(claimsMask),
            "agreemnt mask should be cleared"
        ).to.equal(0);

        // let agreement mask be 00000100
        claimsMask = BigNumber.from(2).pow(256 - 6); // 1 << 250
        claimsMask = await claimsMaskLibrary.clearAgreementMask(claimsMask);
        expect(
            await claimsMaskLibrary.getAgreementMask(claimsMask),
            "again, agreemnt mask should be cleared"
        ).to.equal(0);
    });

    it("set agreement mask", async () => {
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);

        // revert when index is greater than 7
        let validatorIndex = 8;
        await expect(
            claimsMaskLibrary.setAgreementMask(claimsMask, validatorIndex),
            "fail to set agreement mask"
        ).to.be.revertedWith("index out of range");

        // set agreement mask from index 0 to 7
        for (validatorIndex = 0; validatorIndex < 8; validatorIndex++) {
            claimsMask = await claimsMaskLibrary.setAgreementMask(
                claimsMask,
                validatorIndex
            );
            expect(
                await claimsMaskLibrary.getAgreementMask(claimsMask),
                "check agreement mask"
            ).to.equal((1 << (validatorIndex + 1)) - 1);
        }

        // set agreement mask to be 00000100
        claimsMask = await claimsMaskLibrary.newClaimsMask(0);
        claimsMask = await claimsMaskLibrary.setAgreementMask(claimsMask, 2);
        expect(
            await claimsMaskLibrary.getAgreementMask(claimsMask),
            "agreemnt mask should be set to 00000100"
        ).to.equal(4);
    });

    it("get consensus goal mask", async () => {
        // let consensus goal mask initially be 11111111
        let claimsMask = ethers.constants.MaxUint256;
        claimsMask = claimsMask.div(2 ** 8); // empty agreement mask
        // initial consensus goal mask should be all 1, each time right shift 1
        for (let i = 8; i > 0; i--) {
            expect(
                await claimsMaskLibrary.getConsensusGoalMask(claimsMask),
                "initial consensus goal mask should be all 1, each time right shift 1"
            ).to.equal((1 << i) - 1);
            claimsMask = claimsMask.div(2); // claimsMask >> 1
        }

        // let consensus goal mask be 00000100
        claimsMask = BigNumber.from(2).pow(256 - 6 - 8); // 1 << (250-8) minus agreement mask
        expect(
            await claimsMaskLibrary.getConsensusGoalMask(claimsMask),
            "consensus goal mask should be 00000100"
        ).to.equal(4);
    });

    it("remove validator", async () => {
        let claimsMask = await claimsMaskLibrary.newClaimsMask(0);

        // revert when index is greater than 7
        let validatorIndex = 8;
        await expect(
            claimsMaskLibrary.removeValidator(claimsMask, validatorIndex),
            "fail to remove validator"
        ).to.be.revertedWith("index out of range");

        // let ClaimsMask initially be all 1
        claimsMask = ethers.constants.MaxUint256;
        let agreementMask = (1 << 8) - 1;
        let consensusGoalMask = (1 << 8) - 1;
        let otherMask = BigNumber.from(2).pow(240).sub(1); // (1<<240)-1

        // case 1: remove validator one by one, from 7 all the way to 0
        for (validatorIndex = 7; validatorIndex >= 0; validatorIndex--) {
            claimsMask = await claimsMaskLibrary.removeValidator(
                claimsMask,
                validatorIndex
            );

            // simulate each part of mask has a validator removed
            agreementMask = agreementMask >> 1;
            consensusGoalMask = consensusGoalMask >> 1;
            otherMask = otherMask.div(2 ** 30); //otherMask >> 30
            // maskToBe = (agreementMask<<248) | (consensusGoalMask<< 240) | otherMask
            let maskToBe = BigNumber.from(agreementMask)
                .mul(BigNumber.from(2).pow(248))
                .add(
                    BigNumber.from(consensusGoalMask).mul(
                        BigNumber.from(2).pow(240)
                    )
                )
                .add(otherMask);

            expect(claimsMask, "remove validator from 7 to 0").to.equal(
                maskToBe
            );
        }

        // case 2: remove only validator_2
        claimsMask = ethers.constants.MaxUint256;
        validatorIndex = 2;
        claimsMask = await claimsMaskLibrary.removeValidator(
            claimsMask,
            validatorIndex
        );

        let maskToBe = BigNumber.from(ethers.constants.MaxUint256)
            .sub(BigNumber.from(2).pow(250))
            .sub(BigNumber.from(2).pow(250 - 8))
            .sub(
                BigNumber.from(2)
                    .pow(30 * 3)
                    .sub(BigNumber.from(2).pow(30 * 2))
            );
        expect(claimsMask, "remove only validator_2").to.equal(maskToBe);

        // case 3: claimsMask only has validator_2, remove it
        claimsMask = BigNumber.from(2).pow(30 * 2 + 3); // random mask for validator_2
        validatorIndex = 2;
        claimsMask = await claimsMaskLibrary.removeValidator(
            claimsMask,
            validatorIndex
        );
        expect(claimsMask, "now ClaimsMask should be empty").to.equal(0);
    });
});
