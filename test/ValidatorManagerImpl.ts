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
import { deployments, ethers } from "hardhat";
import { solidity, MockProvider } from "ethereum-waffle";
import { ValidatorManagerImpl__factory } from "../src/types/factories/ValidatorManagerImpl__factory";
import { Signer } from "ethers";
import { ValidatorManagerImpl } from "../src/types/ValidatorManagerImpl";

use(solidity);

describe("Validator Manager Implementation", async () => {
    var descartesV2: Signer;
    var signer: Signer;
    var VMI: ValidatorManagerImpl;
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
        await deployments.fixture();
        [descartesV2, signer] = await ethers.getSigners();
        const vmiFactory = new ValidatorManagerImpl__factory(descartesV2);
        var address: any;

        var wallets = provider.getWallets();
        validators = [];

        // add all wallets as validators
        for (var wallet of wallets) {
            address = await wallet.getAddress();
            validators.push(address);
        }

        VMI = await vmiFactory.deploy(
            await descartesV2.getAddress(),
            validators
        );
    });

    it("check initial consensusGoalMask", async () => {
        let initConsensusGoalMask = (1 << validators.length) - 1;
        expect(
            await VMI.getConsensusGoalMask(),
            "get initial consensusGoalMask"
        ).to.equal(initConsensusGoalMask);
    });

    it("check initial claimAgreementMask", async () => {
        expect(
            await VMI.getCurrentAgreementMask(),
            "get initial claimAgreementMask"
        ).to.equal(0);
    });

    it("check initial currentClaim", async () => {
        expect(
            await VMI.getCurrentClaim(),
            "get initial currentClaim"
        ).to.equal(hash_zero);
    });

    it("onClaim and onDisputeEnd should revert if not called from DescartesV2", async () => {
        await expect(
            VMI.connect(signer).onClaim(validators[0], hash_zero),
            "should revert if not called from DescartesV2"
        ).to.be.revertedWith("Only descartesV2");

        await expect(
            VMI.connect(signer).onDisputeEnd(
                address_zero,
                address_zero,
                hash_zero
            ),
            "should revert if not called from DescartesV2"
        ).to.be.revertedWith("Only descartesV2");
    });

    it("onClaim should revert if claim is 0x00", async () => {
        await expect(
            VMI.onClaim(validators[0], hash_zero),
            "should revert if claim == 0x00"
        ).to.be.revertedWith("empty claim");
    });

    it("onClaim should revert if sender is not allowed", async () => {
        var claim = "0x" + "1".repeat(64);
        await expect(
            VMI.onClaim(address_zero, claim),
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
                    await VMI.callStatic.onClaim(validators[i], claim)
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
                VMI.onClaim(validators[i], claim),
                "equal claims should not generate conflict nor consensus, if not all validators have agreed"
            )
                .to.emit(VMI, "ClaimReceived")
                .withArgs(
                    Result.NoConflict,
                    [hash_zero, hash_zero],
                    [address_zero, address_zero]
                );

            // check updated currentAgreementMask
            currentAgreementMask = currentAgreementMask | (1 << i);
            expect(
                await VMI.getCurrentAgreementMask(),
                "check currentAgreementMask"
            ).to.equal(currentAgreementMask);

            // check updated currentClaim
            expect(
                await VMI.getCurrentClaim(),
                "get updated currentClaim"
            ).to.equal(claim);
        }

        // when last validator agrees, should return consensus
        // callStatic: check return value
        var lastValidator = validators[validators.length - 1];
        expect(
            JSON.stringify(await VMI.callStatic.onClaim(lastValidator, claim)),
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
            VMI.onClaim(lastValidator, claim),
            "after all validators claim should be consensus"
        )
            .to.emit(VMI, "ClaimReceived")
            .withArgs(
                Result.Consensus,
                [claim, hash_zero],
                [lastValidator, address_zero]
            );

        // check updated currentAgreementMask
        currentAgreementMask =
            currentAgreementMask | (1 << (validators.length - 1));
        expect(
            await VMI.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);
    });

    it("onClaim with different claims should return conflict", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);

        await expect(
            VMI.onClaim(validators[0], claim),
            "first claim should not generate conflict"
        )
            .to.emit(VMI, "ClaimReceived")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // callStatic: check return value
        expect(
            JSON.stringify(await VMI.callStatic.onClaim(validators[1], claim2)),
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
            VMI.onClaim(validators[1], claim2),
            "different claim should generate conflict"
        )
            .to.emit(VMI, "ClaimReceived")
            .withArgs(
                Result.Conflict,
                [claim, claim2],
                [validators[0], validators[1]]
            );

        // check currentAgreementMask
        var currentAgreementMask = 1;
        expect(
            await VMI.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);
    });

    it("onDisputeEnd with no conflict after", async () => {
        var claim = "0x" + "1".repeat(64);

        // start with no conflict claim to populate
        // variables
        await VMI.onClaim(validators[0], claim);

        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onDisputeEnd(
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
            VMI.onDisputeEnd(validators[0], validators[1], claim),
            "if winning claim is current claim and there is no consensus, should return NoConflict"
        )
            .to.emit(VMI, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // check currentAgreementMask
        var currentAgreementMask = 1;
        expect(
            await VMI.getCurrentAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);

        // check consensusGoalMask
        // consensusGoalMask should remove loser validators[1]
        var consensusGoalMask = (1 << validators.length) - 1 - (1 << 1);
        expect(
            await VMI.getConsensusGoalMask(),
            "check consensusGoalMask"
        ).to.equal(consensusGoalMask);
    });

    it("onDisputeEnd with consensus after", async () => {
        var claim = "0x" + "1".repeat(64);
        var lastValidator = validators[validators.length - 1];

        // all validators agree but last one
        for (var i = 0; i < validators.length - 1; i++) {
            await VMI.onClaim(validators[i], claim);
        }

        // last validator lost dispute, the only one that disagreed
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onDisputeEnd(
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
            VMI.onDisputeEnd(validators[0], lastValidator, claim),
            "if losing claim was the only one not agreeing, should return consensus"
        )
            .to.emit(VMI, "DisputeEnded")
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
            await VMI.onClaim(validators[i], claim);
        }
        // first validator lost dispute
        // next defender should be validators[1]
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onDisputeEnd(
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
            VMI.onDisputeEnd(lastValidator, validators[0], claim2),
            "conflict should continue if there are validators still defending claim that lost"
        )
            .to.emit(VMI, "DisputeEnded")
            .withArgs(
                Result.Conflict,
                [claim, claim2],
                [validators[1], lastValidator]
            );

        // make all other validators but last defending the losing dispute
        for (var i = 1; i < validators.length - 2; i++) {
            await VMI.onDisputeEnd(lastValidator, validators[i], claim2);
        }

        // honest validator by himself can generate consensus
        // by winning his last dispute
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onDisputeEnd(
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
            VMI.onDisputeEnd(
                lastValidator,
                validators[validators.length - 2],
                claim2
            ),
            "lastValidator should be the last one in the validator set"
        )
            .to.emit(VMI, "DisputeEnded")
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
            await VMI.onClaim(validators[i], claim);
        }

        // make all other validators but the last two defending the losing dispute
        for (var i = 0; i < validators.length - 3; i++) {
            await VMI.onDisputeEnd(lastValidator, validators[i], claim2);
        }
        // honest validator winning the last dispute
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onDisputeEnd(
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
            VMI.onDisputeEnd(
                lastValidator,
                validators[validators.length - 3],
                claim2
            ),
            "check emitted event for the last dispute"
        )
            .to.emit(VMI, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [hash_zero, hash_zero],
                [address_zero, address_zero]
            );

        // now the second last validator can finalize the consensus
        // callStatic: check return value
        expect(
            JSON.stringify(
                await VMI.callStatic.onClaim(secondLastValidator, claim2)
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
            VMI.onClaim(secondLastValidator, claim2),
            "finalize the consensus"
        )
            .to.emit(VMI, "ClaimReceived")
            .withArgs(
                Result.Consensus,
                [claim2, hash_zero],
                [secondLastValidator, address_zero]
            );
    });

    it("onNewEpoch", async () => {
        var claim = "0x" + "1".repeat(64);

        // one validator claims
        await VMI.onClaim(validators[0], claim);

        // epoch ends without consensus
        // callStatic: check return value
        expect(
            await VMI.callStatic.onNewEpoch(),
            "onNewEpoch() should return current claim"
        ).to.equal(claim);
        // check emitted event
        await expect(
            VMI.onNewEpoch(),
            "new epoch should emit event NewEpoch with current claim"
        )
            .to.emit(VMI, "NewEpoch")
            .withArgs(claim);

        expect(
            await VMI.getCurrentAgreementMask(),
            "current agreement mask should reset"
        ).to.equal(0);

        expect(
            await VMI.getCurrentClaim(),
            "current claim should reset"
        ).to.equal(hash_zero);
    });
});
