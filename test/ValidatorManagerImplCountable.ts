// Copyright (C) 2020 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import { expect, use } from "chai";
import { ethers } from "hardhat";
import { solidity, MockProvider } from "ethereum-waffle";
import { ValidatorManagerImplCountable__factory } from "../src/types/factories/ValidatorManagerImplCountable__factory";
import { Signer } from "ethers";
import { ValidatorManagerImplCountable } from "../src/types/ValidatorManagerImplCountable";

use(solidity);

describe("Validator Manager Implementation Countable", async () => {
    var descartesV2: Signer;
    var signer: Signer;
    var VMIC: ValidatorManagerImplCountable;
    const provider = new MockProvider();
    var validators: string[] = [];

    let hash_zero = ethers.constants.HashZero;
    let address_zero = "0x0000000000000000000000000000000000000000";

    enum Result {
        NoConflict,
        Consensus,
        Conflict,
    }

    beforeEach(async () => {
        [descartesV2, signer] = await ethers.getSigners();
        const vmicFactory = new ValidatorManagerImplCountable__factory(
            descartesV2
        );
        var address: any;

        var wallets = provider.getWallets();
        validators = [];

        // add 8 wallets as 8 validators
        for (let i = 0; i < 8; i++) {
            address = await wallets[i].getAddress();
            validators.push(address);
        }

        VMIC = await vmicFactory.deploy(
            await descartesV2.getAddress(),
            validators
        );
    });

    it("should revert if more than 8 validators", async () => {
        var wallets = provider.getWallets();
        var wrongValidators = [];
        for (let i = 0; i < 10; i++) {
            var address = await wallets[i].getAddress();
            wrongValidators.push(address);
        }
        const vmicFactory = new ValidatorManagerImplCountable__factory(
            descartesV2
        );
        await expect(
            vmicFactory.deploy(await descartesV2.getAddress(), wrongValidators)
        ).to.be.revertedWith("up to 8 validators");
    });

    it("check initial consensusGoalMask", async () => {
        let initConsensusGoalMask = (1 << validators.length) - 1;
        expect(
            await VMIC.getConsensusGoalMask(),
            "get initial consensusGoalMask"
        ).to.equal(initConsensusGoalMask);
    });

    it("check initial claimAgreementMask", async () => {
        expect(
            await VMIC.getCurrentAgreementMask(),
            "get initial claimAgreementMask"
        ).to.equal(0);
    });

    it("check initial currentClaim", async () => {
        expect(
            await VMIC.getCurrentClaim(),
            "get initial currentClaim"
        ).to.equal(hash_zero);
    });

    it("onClaim and onDisputeEnd should revert if not called from DescartesV2", async () => {
        await expect(
            VMIC.connect(signer).onClaim(validators[0], hash_zero),
            "should revert if not called from DescartesV2"
        ).to.be.revertedWith("Only descartesV2");

        await expect(
            VMIC.connect(signer).onDisputeEnd(
                address_zero,
                address_zero,
                hash_zero
            ),
            "should revert if not called from DescartesV2"
        ).to.be.revertedWith("Only descartesV2");
    });

    it("onClaim should revert if claim is 0x00", async () => {
        await expect(
            VMIC.onClaim(validators[0], hash_zero),
            "should revert if claim == 0x00"
        ).to.be.revertedWith("empty claim");
    });

    it("onClaim should revert if sender is not allowed", async () => {
        var claim = "0x" + "1".repeat(64);
        await expect(
            VMIC.onClaim(address_zero, claim),
            "should revert if sender is not in validators array"
        ).to.be.revertedWith("sender not allowed");
    });

    it("onClaim NoConflict and Consensus", async () => {
        var claim = "0x" + "1".repeat(64);
        var currentAgreementMask = 0;

        // if validators keep agreeing there is no conflict
        for (var i = 0; i < validators.length - 1; i++) {
            // callStatic: check return value
            expect(
                JSON.stringify(
                    await VMIC.callStatic.onClaim(validators[i], claim)
                ),
                "use callStatic to check return value of onClaim() when NoConflict"
            ).to.equal(
                JSON.stringify([
                    Result.NoConflict,
                    [hash_zero, hash_zero],
                    [address_zero, address_zero],
                ])
            );

            // check emitted event
            await expect(
                VMIC.onClaim(validators[i], claim),
                "equal claims should not generate conflict nor consensus, if not all validators have agreed"
            )
                .to.emit(VMIC, "ClaimReceived")
                .withArgs(
                    Result.NoConflict,
                    [hash_zero, hash_zero],
                    [address_zero, address_zero]
                );

            // check updated currentAgreementMask
            currentAgreementMask = currentAgreementMask | (1 << i);
            expect(
                await VMIC.getCurrentAgreementMask(),
                "check currentAgreementMask"
            ).to.equal(currentAgreementMask);

            // check updated currentClaim
            expect(
                await VMIC.getCurrentClaim(),
                "get updated currentClaim"
            ).to.equal(claim);
        }

        // when last validator agrees, should return consensus
        // callStatic: check return value
        var lastValidator = validators[validators.length - 1];
        expect(
            JSON.stringify(await VMIC.callStatic.onClaim(lastValidator, claim)),
            "use callStatic to check return value of onClaim() when Consensus"
        ).to.equal(
            JSON.stringify([
                Result.Consensus,
                [claim, hash_zero],
                [lastValidator, address_zero],
            ])
        );

        // check emitted event
        await expect(
            VMIC.onClaim(lastValidator, claim),
            "after all validators claim should be consensus"
        )
            .to.emit(VMIC, "ClaimReceived")
            .withArgs(
                Result.Consensus,
                [claim, hash_zero],
                [lastValidator, address_zero]
            );

        // check updated currentAgreementMask
        currentAgreementMask =
            currentAgreementMask | (1 << (validators.length - 1));
        expect(
            await VMIC.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);
    });

    it("onClaim with different claims should return conflict", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);

        await expect(
            VMIC.onClaim(validators[0], claim),
            "first claim should not generate conflict"
        )
            .to.emit(VMIC, "ClaimReceived")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onClaim(validators[1], claim2)
            ),
            "use callStatic to check return value of onClaim() when conflict"
        ).to.equal(
            JSON.stringify([
                Result.Conflict,
                [claim, claim2],
                [validators[0], validators[1]],
            ])
        );

        // check emitted event
        await expect(
            VMIC.onClaim(validators[1], claim2),
            "different claim should generate conflict"
        )
            .to.emit(VMIC, "ClaimReceived")
            .withArgs(
                Result.Conflict,
                [claim, claim2],
                [validators[0], validators[1]]
            );

        // check currentAgreementMask
        var currentAgreementMask = 1;
        expect(
            await VMIC.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);
    });

    it("onDisputeEnd with no conflict after", async () => {
        var claim = "0x" + "1".repeat(64);

        // start with no conflict claim to populate
        // variables
        await VMIC.onClaim(validators[0], claim);

        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onDisputeEnd(
                    validators[0],
                    validators[1],
                    claim
                )
            ),
            "use callStatic to check return value of onDisputeEnd() when no conflict after"
        ).to.equal(
            JSON.stringify([
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero],
            ])
        );

        // check emitted event
        await expect(
            VMIC.onDisputeEnd(validators[0], validators[1], claim),
            "if winning claim is current claim and there is no consensus, should return NoConflict"
        )
            .to.emit(VMIC, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // check currentAgreementMask
        var currentAgreementMask = 1;
        expect(
            await VMIC.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);

        // check consensusGoalMask
        // consensusGoalMask should remove loser validators[1]
        var consensusGoalMask = (1 << validators.length) - 1 - (1 << 1);
        expect(
            await VMIC.getConsensusGoalMask(),
            "check consensusGoalMask"
        ).to.equal(consensusGoalMask);
    });

    it("onDisputeEnd with consensus after", async () => {
        var claim = "0x" + "1".repeat(64);
        var lastValidator = validators[validators.length - 1];

        // all validators agree but last one
        for (var i = 0; i < validators.length - 1; i++) {
            await VMIC.onClaim(validators[i], claim);
        }

        // last validator lost dispute, the only one that disagreed
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onDisputeEnd(
                    validators[0],
                    lastValidator,
                    claim
                )
            ),
            "use callStatic to check return value of onDisputeEnd() when consensus after"
        ).to.equal(
            JSON.stringify([
                Result.Consensus,
                [claim, hash_zero],
                [validators[0], address_zero],
            ])
        );

        // check emitted event
        await expect(
            VMIC.onDisputeEnd(validators[0], lastValidator, claim),
            "if losing claim was the only one not agreeing, should return consensus"
        )
            .to.emit(VMIC, "DisputeEnded")
            .withArgs(
                Result.Consensus,
                [claim, hash_zero],
                [validators[0], address_zero]
            );
    });

    it("onDisputeEnd multiple validators defending lost claim", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        var lastValidator = validators[validators.length - 1];

        // all validators agree but last one
        for (var i = 0; i < validators.length - 1; i++) {
            await VMIC.onClaim(validators[i], claim);
        }
        // first validator lost dispute
        // next defender should be validators[1]
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onDisputeEnd(
                    lastValidator,
                    validators[0],
                    claim2
                )
            ),
            "use callStatic to check return value of onDisputeEnd() after first dispute"
        ).to.equal(
            JSON.stringify([
                Result.Conflict,
                [claim, claim2],
                [validators[1], lastValidator],
            ])
        );
        // check emitted event
        await expect(
            VMIC.onDisputeEnd(lastValidator, validators[0], claim2),
            "conflict should continue if there are validators still defending claim that lost"
        )
            .to.emit(VMIC, "DisputeEnded")
            .withArgs(
                Result.Conflict,
                [claim, claim2],
                [validators[1], lastValidator]
            );

        // make all other validators but last defending the losing dispute
        for (var i = 1; i < validators.length - 2; i++) {
            await VMIC.onDisputeEnd(lastValidator, validators[i], claim2);
        }

        // honest validator by himself can generate consensus
        // by winning his last dispute
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onDisputeEnd(
                    lastValidator,
                    validators[validators.length - 2],
                    claim2
                )
            ),
            "use callStatic to check return value of onDisputeEnd() after last dispute"
        ).to.equal(
            JSON.stringify([
                Result.Consensus,
                [claim2, hash_zero],
                [lastValidator, address_zero],
            ])
        );
        // check emitted event
        await expect(
            VMIC.onDisputeEnd(
                lastValidator,
                validators[validators.length - 2],
                claim2
            ),
            "lastValidator should be the last one in the validator set"
        )
            .to.emit(VMIC, "DisputeEnded")
            .withArgs(
                Result.Consensus,
                [claim2, hash_zero],
                [lastValidator, address_zero]
            );
    });

    it("onDisputeEnd validators but the last two defending lost claim", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        var lastValidator = validators[validators.length - 1];
        var secondLastValidator = validators[validators.length - 2];

        // all validators agree but the last two
        for (var i = 0; i < validators.length - 2; i++) {
            await VMIC.onClaim(validators[i], claim);
        }

        // make all other validators but the last two defending the losing dispute
        for (var i = 0; i < validators.length - 3; i++) {
            await VMIC.onDisputeEnd(lastValidator, validators[i], claim2);
        }
        // honest validator winning the last dispute
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onDisputeEnd(
                    lastValidator,
                    validators[validators.length - 3],
                    claim2
                )
            ),
            "use callStatic to check return value of onDisputeEnd() after last dispute"
        ).to.equal(
            JSON.stringify([
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero],
            ])
        );
        // check emitted event
        await expect(
            VMIC.onDisputeEnd(
                lastValidator,
                validators[validators.length - 3],
                claim2
            ),
            "check emitted event for the last dispute"
        )
            .to.emit(VMIC, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // now the second last validator can finalize the consensus
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMIC.callStatic.onClaim(secondLastValidator, claim2)
            ),
            "use callStatic to check return value of onClaim() to finalize consensus"
        ).to.equal(
            JSON.stringify([
                Result.Consensus,
                [claim2, hash_zero],
                [secondLastValidator, address_zero],
            ])
        );
        // check emitted event
        await expect(
            VMIC.onClaim(secondLastValidator, claim2),
            "finalize the consensus"
        )
            .to.emit(VMIC, "ClaimReceived")
            .withArgs(
                Result.Consensus,
                [claim2, hash_zero],
                [secondLastValidator, address_zero]
            );
    });

    it("onNewEpoch", async () => {
        var claim = "0x" + "1".repeat(64);

        // one validator claims
        await VMIC.onClaim(validators[0], claim);

        // epoch ends without consensus
        // callStatic: check return value
        expect(
            await VMIC.callStatic.onNewEpoch(),
            "onNewEpoch() should return current claim"
        ).to.equal(claim);
        // check emitted event
        await expect(
            VMIC.onNewEpoch(),
            "new epoch should emit event NewEpoch with current claim"
        )
            .to.emit(VMIC, "NewEpoch")
            .withArgs(claim);

        expect(
            await VMIC.getCurrentAgreementMask(),
            "current agreement mask should reset"
        ).to.equal(0);

        expect(
            await VMIC.getCurrentClaim(),
            "current claim should reset"
        ).to.equal(hash_zero);
    });

    it("test #claims", async () => {
        // check initial #claims
        for (var i = 0; i < validators.length; i++) {
            expect(
                await VMIC.getNumberOfClaimsByAddress(validators[i]),
                "initial #claims"
            ).to.equal(0);
        }

        // all validators make the same claim
        var claim = "0x" + "1".repeat(64);
        for (var i = 0; i < validators.length; i++) {
            await VMIC.onClaim(validators[i], claim);
            expect(
                await VMIC.getNumberOfClaimsByAddress(validators[i]),
                "still 0 because consensus hasn't reached"
            ).to.equal(0);
        }
        // once consensus, #claims increases
        await VMIC.onNewEpoch();
        for (var i = 0; i < validators.length; i++) {
            expect(
                await VMIC.getNumberOfClaimsByAddress(validators[i]),
                "now #claims increased"
            ).to.equal(1);
        }

        // keep skipping to new epoches
        for (let epoch = 1; epoch < 20; epoch++) {
            // 1st validator keeps making claims
            await VMIC.onClaim(validators[0], claim);
            await VMIC.onNewEpoch();
            // check how #claims is increasing
            expect(
                await VMIC.getNumberOfClaimsByAddress(validators[0]),
                "check increasing #claims"
            ).to.equal(epoch + 1);
        }

        // currently, #claims gets cleared once a validator makes a wrong claim
        await VMIC.onClaim(validators[0], claim);
        var claim2 = "0x" + "2".repeat(64);
        await VMIC.onClaim(validators[1], claim2);
        // let the 2nd validator win the dispute
        await VMIC.onDisputeEnd(validators[1], validators[0], claim2);
        await VMIC.onNewEpoch();
        expect(
            await VMIC.getNumberOfClaimsByAddress(validators[0]),
            "now the #claims for validator0 should get cleared"
        ).to.equal(0);
        expect(
            await VMIC.getNumberOfClaimsByAddress(validators[1]),
            "#claims for validator1 should increase by 1"
        ).to.equal(2);
    });
});
