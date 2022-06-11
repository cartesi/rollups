// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { expect, use } from "chai";
import { deployments, ethers } from "hardhat";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    DebugFacet,
    DebugFacet__factory,
    RollupsFacet,
    RollupsFacet__factory,
    ValidatorManagerFacet,
    ValidatorManagerFacet__factory,
} from "../src/types";
import { deployDiamond, getState, increaseTimeAndMine } from "./utils";

use(solidity);

describe("Validator Manager Facet", async () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    var signers: Signer[];
    var rollupsFacet: RollupsFacet;
    var validatorManagerFacet: ValidatorManagerFacet;
    var debugFacet: DebugFacet;
    var validators: string[] = [];
    let inputDuration: number;
    let challengePeriod: number;

    let hash_zero = ethers.constants.HashZero;
    let address_zero = "0x0000000000000000000000000000000000000000";
    let address_one = "0x0000000000000000000000000000000000000001";

    enum Result {
        NoConflict,
        Consensus,
        Conflict,
    }

    let initialState: string; // for delegate

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

    beforeEach(async () => {
        await deployments.fixture();

        signers = await ethers.getSigners();
        const diamond = await deployDiamond({ debug: true });
        rollupsFacet = RollupsFacet__factory.connect(
            diamond.address,
            signers[0]
        );
        validatorManagerFacet = ValidatorManagerFacet__factory.connect(
            diamond.address,
            signers[0]
        );
        debugFacet = DebugFacet__factory.connect(diamond.address, signers[0]);
        validators = await debugFacet._getValidators();
        inputDuration = (await rollupsFacet.getInputDuration()).toNumber();
        challengePeriod = (await rollupsFacet.getChallengePeriod()).toNumber();

        // initial state for delegate
        initialState = JSON.stringify({
            dapp_contract_address: validatorManagerFacet.address,
        });
    });

    it("check initial consensusGoalMask", async () => {
        let initConsensusGoalMask = (1 << validators.length) - 1;
        expect(
            await validatorManagerFacet.getConsensusGoalMask(),
            "get initial consensusGoalMask"
        ).to.equal(initConsensusGoalMask);
    });

    it("check initial claimAgreementMask", async () => {
        expect(
            await validatorManagerFacet.getAgreementMask(),
            "get initial claimAgreementMask"
        ).to.equal(0);
    });

    it("check initial currentClaim", async () => {
        expect(
            await validatorManagerFacet.getCurrentClaim(),
            "get initial currentClaim"
        ).to.equal(hash_zero);
    });

    it("check initial maximum number of validators", async () => {
        expect(
            await validatorManagerFacet.getMaxNumValidators(),
            "get maximum number of validators"
        ).to.equal(validators.length);
    });

    if (enableDelegate) {
        it("check initial delegate", async () => {
            let state = JSON.parse(await getState(initialState));

            // check `num_claims` in delegate
            // in delegates, `num_claims` is hard-coded to have 8 Options
            expect(state.num_claims.length, "should have 8 Options").to.equal(
                8
            );
            for (let i = 0; i < 8; i++) {
                expect(
                    state.num_claims[i],
                    "each Option should be null initially"
                ).to.equal(null);
            }

            // check `claiming` in delegate
            expect(
                state.claiming.length,
                "`claiming` should be empty initially"
            ).to.equal(0);

            // check `validators_removed` in delegate
            expect(
                state.validators_removed.length,
                "`validators_removed` should be empty initially"
            ).to.equal(0);

            // check `num_finalized_epochs` in delegate
            expect(
                state.num_finalized_epochs,
                "initial epoch should be 0"
            ).to.equal("0x0");

            // check `dapp_contract_address` in delegate
            expect(
                state.dapp_contract_address,
                "dapp contract address should be same as validator manager facet address"
            ).to.equal(validatorManagerFacet.address.toLowerCase());
        });
    }

    it("onClaim should revert if claim is 0x00", async () => {
        await expect(
            debugFacet._onClaim(validators[0], hash_zero),
            "should revert if claim == 0x00"
        ).to.be.revertedWith("empty claim");
    });

    it("onClaim should revert if sender is address 0", async () => {
        var claim = "0x" + "1".repeat(64);
        await expect(
            debugFacet._onClaim(address_zero, claim),
            "should revert if sender is address 0"
        ).to.be.revertedWith("address 0");
    });

    it("onClaim should revert if sender is not allowed", async () => {
        var claim = "0x" + "1".repeat(64);
        await expect(
            debugFacet._onClaim(address_one, claim),
            "should revert if sender is not in validators array"
        ).to.be.revertedWith("sender not allowed");
    });

    it("onClaim should revert if sender has already claimed before", async () => {
        var claim = "0x" + "1".repeat(64);
        for (var i = 0; i < validators.length; i++) {
            // let validators all make a claim
            await debugFacet._onClaim(validators[i], claim);

            // they should not be able to claim again in the same epoch
            await expect(
                debugFacet._onClaim(validators[i], claim),
                "should revert since validator has already claimed before"
            ).to.be.revertedWith("sender had claimed in this epoch before");
        }

        // after entering into a new epoch, validators should be able to claim again
        await debugFacet._onNewEpochVM();
        for (var i = 0; i < validators.length; i++) {
            await debugFacet._onClaim(validators[i], claim);
        }
    });

    it("onClaim NoConflict and Consensus", async () => {
        var claim = "0x" + "1".repeat(64);
        var currentAgreementMask = 0;
        if (!enableDelegate) {
            // if validators keep agreeing there is no conflict
            for (var i = 0; i < validators.length - 1; i++) {
                // callStatic: check return value
                expect(
                    JSON.stringify(
                        await debugFacet.callStatic._onClaim(
                            validators[i],
                            claim
                        )
                    ),
                    "use callStatic to check return value of onClaim() when NoConflict"
                ).to.equal(
                    JSON.stringify([
                        Result.NoConflict,
                        [claim, hash_zero],
                        [validators[i], address_zero],
                    ])
                );

                // check emitted event
                await expect(
                    debugFacet._onClaim(validators[i], claim),
                    "equal claims should not generate conflict nor consensus, if not all validators have agreed"
                )
                    .to.emit(debugFacet, "ClaimReceived")
                    .withArgs(
                        Result.NoConflict,
                        [claim, hash_zero],
                        [validators[i], address_zero]
                    );

                // check updated currentAgreementMask
                currentAgreementMask = currentAgreementMask | (1 << i);
                expect(
                    await validatorManagerFacet.getAgreementMask(),
                    "check currentAgreementMask"
                ).to.equal(currentAgreementMask);

                // check updated currentClaim
                expect(
                    await validatorManagerFacet.getCurrentClaim(),
                    "get updated currentClaim"
                ).to.equal(claim);
            }

            // when last validator agrees, should return consensus
            // callStatic: check return value
            var lastValidator = validators[validators.length - 1];
            expect(
                JSON.stringify(
                    await debugFacet.callStatic._onClaim(lastValidator, claim)
                ),
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
                debugFacet._onClaim(lastValidator, claim),
                "after all validators claim should be consensus"
            )
                .to.emit(debugFacet, "ClaimReceived")
                .withArgs(
                    Result.Consensus,
                    [claim, hash_zero],
                    [lastValidator, address_zero]
                );

            // check updated currentAgreementMask
            currentAgreementMask =
                currentAgreementMask | (1 << (validators.length - 1));
            expect(
                await validatorManagerFacet.getAgreementMask(),
                "check currentAgreementMask"
            ).to.equal(currentAgreementMask);
        } else {
            // test delegate
            for (var i = 0; i < validators.length - 1; i++) {
                await passInputAccumulationPeriod();
                await rollupsFacet.connect(signers[i]).claim(claim);
                let state = JSON.parse(await getState(initialState));

                // each round there should be 1 more validator in `claiming`
                for (var j = 0; j <= i; j++) {
                    expect(
                        state.claiming[j],
                        "check validators in `claiming`"
                    ).to.equal((await signers[j].getAddress()).toLowerCase());
                }

                // the rest should remain as initial
                expect(
                    state.num_claims.length,
                    "should have 8 Options"
                ).to.equal(8);
                for (let i = 0; i < 8; i++) {
                    expect(
                        state.num_claims[i],
                        "each Option should be null initially"
                    ).to.equal(null);
                }
                expect(
                    state.validators_removed.length,
                    "`validators_removed` should be empty initially"
                ).to.equal(0);
                expect(
                    state.num_finalized_epochs,
                    "initial epoch should be 0"
                ).to.equal("0x0");
            }

            // now the last validator claims
            // since we use `claim()` function from Rollups,
            // validator manager's `onNewEpoch()` will be called when there's Consensus
            await rollupsFacet.connect(signers[7]).claim(claim); // Consensus

            let state = JSON.parse(await getState(initialState));
            // now enters new epoch and validators in `claiming` will be cleared
            // and their `num_claims` will increase
            expect(state.claiming.length, "`claiming` is cleared").to.equal(0);
            for (let i = 0; i < 8; i++) {
                expect(
                    state.num_claims[i].validator_address,
                    "`num_claims` should have included the validator"
                ).to.equal((await signers[i].getAddress()).toLowerCase());
                expect(
                    state.num_claims[i].num_claims_made,
                    "`num_claims` should be 1 for each validator"
                ).to.equal("0x1");
            }

            // now the number of finalized epochs should be 1
            expect(state.num_finalized_epochs, "1 finalized epoch").to.equal(
                "0x1"
            );

            // the rest should be empty
            expect(
                state.claiming.length,
                "`claiming` should be empty"
            ).to.equal(0);
            expect(
                state.validators_removed.length,
                "`validators_removed` should be empty"
            ).to.equal(0);
        }
    });

    it("onClaim with different claims should return conflict", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        if (!enableDelegate) {
            await expect(
                debugFacet._onClaim(validators[0], claim),
                "first claim should not generate conflict"
            )
                .to.emit(debugFacet, "ClaimReceived")
                .withArgs(
                    Result.NoConflict,
                    [claim, hash_zero],
                    [validators[0], address_zero]
                );

            // callStatic: check return value
            expect(
                JSON.stringify(
                    await debugFacet.callStatic._onClaim(validators[1], claim2)
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
                debugFacet._onClaim(validators[1], claim2),
                "different claim should generate conflict"
            )
                .to.emit(debugFacet, "ClaimReceived")
                .withArgs(
                    Result.Conflict,
                    [claim, claim2],
                    [validators[0], validators[1]]
                );

            // check currentAgreementMask
            var currentAgreementMask = 1;
            expect(
                await validatorManagerFacet.getAgreementMask(),
                "check currentAgreementMask"
            ).to.equal(currentAgreementMask);
        } else {
            // test delegate
            await passInputAccumulationPeriod();
            await rollupsFacet.connect(signers[0]).claim(claim);
            await rollupsFacet.connect(signers[1]).claim(claim2); // conflict
            let state = JSON.parse(await getState(initialState));
            // signers[0] wins the disputes, will be in `claiming`
            expect(
                state.claiming.length,
                "only 1 validator in the `claiming`"
            ).to.equal(1);
            expect(
                state.claiming[0],
                "only signers[0] in the `claiming`"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            // signers[1] loses the disputes, will be in `validators_removed`
            expect(
                state.validators_removed.length,
                "only 1 validator in the `validators_removed`"
            ).to.equal(1);
            expect(
                state.validators_removed[0],
                "only signers[1] in the `validators_removed`"
            ).to.equal((await signers[1].getAddress()).toLowerCase());

            // the rest should be the same as initial
            // `num_claims`
            expect(state.num_claims.length, "should have 8 Options").to.equal(
                8
            );
            for (let i = 0; i < 8; i++) {
                expect(
                    state.num_claims[i],
                    "each Option should be null"
                ).to.equal(null);
            }
            // `num_finalized_epochs`
            expect(state.num_finalized_epochs, "epoch should be 0").to.equal(
                "0x0"
            );
        }
    });

    it("onDisputeEnd with no conflict after", async () => {
        // delegate test for this is the same as the previous one
        // namely the "onClaim with different claims should return conflict"

        var claim = "0x" + "1".repeat(64);

        // start with no conflict claim to populate
        // variables
        await debugFacet._onClaim(validators[0], claim);

        // callStatic: check return value
        expect(
            JSON.stringify(
                await debugFacet.callStatic._onDisputeEnd(
                    validators[0],
                    validators[1],
                    claim
                )
            ),
            "use callStatic to check return value of onDisputeEnd() when no conflict after"
        ).to.equal(
            JSON.stringify([
                Result.NoConflict,
                [claim, hash_zero],
                [validators[0], address_zero],
            ])
        );

        // check emitted event
        await expect(
            debugFacet._onDisputeEnd(validators[0], validators[1], claim),
            "if winning claim is current claim and there is no consensus, should return NoConflict"
        )
            .to.emit(debugFacet, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [claim, hash_zero],
                [validators[0], address_zero]
            );

        // check currentAgreementMask
        var currentAgreementMask = 1;
        expect(
            await validatorManagerFacet.getAgreementMask(),
            "check currentAgreementMask"
        ).to.equal(currentAgreementMask);

        // check consensusGoalMask
        // consensusGoalMask should remove loser validators[1]
        var consensusGoalMask = (1 << validators.length) - 1 - (1 << 1);
        expect(
            await validatorManagerFacet.getConsensusGoalMask(),
            "check consensusGoalMask"
        ).to.equal(consensusGoalMask);
    });

    it("onDisputeEnd with consensus after", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        var lastValidator = validators[validators.length - 1];

        if (!enableDelegate) {
            // all validators agree but last one
            for (var i = 0; i < validators.length - 1; i++) {
                await debugFacet._onClaim(validators[i], claim);
            }

            // last validator lost dispute, the only one that disagreed
            // callStatic: check return value
            expect(
                JSON.stringify(
                    await debugFacet.callStatic._onDisputeEnd(
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
                debugFacet._onDisputeEnd(validators[0], lastValidator, claim),
                "if losing claim was the only one not agreeing, should return consensus"
            )
                .to.emit(debugFacet, "DisputeEnded")
                .withArgs(
                    Result.Consensus,
                    [claim, hash_zero],
                    [validators[0], address_zero]
                );
        } else {
            // test delegate
            await passInputAccumulationPeriod();
            for (let i = 0; i < 8 - 1; i++) {
                await rollupsFacet.connect(signers[i]).claim(claim);
            }
            // conflict and then consensus right after
            await rollupsFacet.connect(signers[7]).claim(claim2);
            let state = JSON.parse(await getState(initialState));

            // since there was a consensus, 7 validators who claimed correctly
            // are added to `num_claims`
            for (let i = 0; i < 8 - 1; i++) {
                expect(
                    state.num_claims[i].validator_address,
                    "`num_claims` should have included the validator"
                ).to.equal((await signers[i].getAddress()).toLowerCase());
                expect(
                    state.num_claims[i].num_claims_made,
                    "`num_claims` should be 1 for each validator"
                ).to.equal("0x1");
            }
            expect(
                state.num_claims[7],
                "the last one in `num_claims` should be null"
            ).to.equal(null);

            // signers[7] who lost the dispute should be in `validators_removed`
            expect(
                state.validators_removed.length,
                "only 1 validator in the `validators_removed`"
            ).to.equal(1);
            expect(
                state.validators_removed[0],
                "only signers[7] in the `validators_removed`"
            ).to.equal((await signers[7].getAddress()).toLowerCase());

            // start to enter epoch 1
            expect(
                state.num_finalized_epochs,
                "start to enter epoch 1"
            ).to.equal("0x1");

            // no validator has claimed in epoch 1 yet
            expect(
                state.claiming.length,
                "no validator has claimed in epoch 1 yet"
            ).to.equal(0);
        }
    });

    it("onDisputeEnd multiple validators defending lost claim", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        var lastValidator = validators[validators.length - 1];

        if (!enableDelegate) {
            // all validators agree but last one
            for (var i = 0; i < validators.length - 1; i++) {
                await debugFacet._onClaim(validators[i], claim);
            }
            // first validator lost dispute
            // next defender should be validators[1]
            // callStatic: check return value
            expect(
                JSON.stringify(
                    await debugFacet.callStatic._onDisputeEnd(
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
                debugFacet._onDisputeEnd(lastValidator, validators[0], claim2),
                "conflict should continue if there are validators still defending claim that lost"
            )
                .to.emit(debugFacet, "DisputeEnded")
                .withArgs(
                    Result.Conflict,
                    [claim, claim2],
                    [validators[1], lastValidator]
                );

            // make all other validators but last defending the losing dispute
            for (var i = 1; i < validators.length - 2; i++) {
                await debugFacet._onDisputeEnd(
                    lastValidator,
                    validators[i],
                    claim2
                );
            }

            // honest validator by himself can generate consensus
            // by winning his last dispute
            // callStatic: check return value
            expect(
                JSON.stringify(
                    await debugFacet.callStatic._onDisputeEnd(
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
                debugFacet._onDisputeEnd(
                    lastValidator,
                    validators[validators.length - 2],
                    claim2
                ),
                "lastValidator should be the last one in the validator set"
            )
                .to.emit(debugFacet, "DisputeEnded")
                .withArgs(
                    Result.Consensus,
                    [claim2, hash_zero],
                    [lastValidator, address_zero]
                );
        } else {
            // test delegate
            await passInputAccumulationPeriod();
            // let signers[0] have the correct claim and all others have false claim
            await rollupsFacet.connect(signers[0]).claim(claim);
            for (let i = 1; i < 8 - 1; i++) {
                await rollupsFacet.connect(signers[i]).claim(claim2);
                let state = JSON.parse(await getState(initialState));

                // only signers[0] is in the `claiming`
                expect(
                    state.claiming.length,
                    "should be only 1 in `claiming`"
                ).to.equal(1);
                expect(
                    state.claiming[0],
                    "only signers[0] in `claiming`"
                ).to.equal((await signers[0].getAddress()).toLowerCase());

                // all other validators should be added into `validators_removed`
                // one by one
                for (let j = 0; j < i; j++) {
                    expect(
                        state.validators_removed[j],
                        "others in `validators_removed`"
                    ).to.equal(
                        (await signers[j + 1].getAddress()).toLowerCase()
                    );
                }
            }

            // once the last validator enters a dispute, the current epoch will end
            // only signers[0] will be moved from `claiming` to `num_claims`
            await rollupsFacet.connect(signers[7]).claim(claim2);
            let state = JSON.parse(await getState(initialState));

            expect(state.num_finalized_epochs, "enter epoch 1").to.equal("0x1");

            expect(
                state.claiming.length,
                "`claiming` should be empty"
            ).to.equal(0);

            expect(
                state.num_claims[0].validator_address,
                "only signers[0] is moved to `num_claims`"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                state.num_claims[0].num_claims_made,
                "signers[0] made 1 claim"
            ).to.equal("0x1");
            for (let i = 1; i < 8; i++) {
                expect(
                    state.num_claims[i],
                    "all other fields in `num_claims` should be null"
                ).to.equal(null);
            }

            // all other 7 validators are in `validators_removed`
            for (let i = 1; i < 8; i++) {
                expect(
                    state.validators_removed[i - 1],
                    "all other 7 validators are in `validators_removed`"
                ).to.equal((await signers[i].getAddress()).toLowerCase());
            }
        }
    });

    it("onDisputeEnd validators but the last two defending lost claim", async () => {
        // no delegate test here because we can't control who wins the dispute in delegates
        // but the previous delegate test should have the same effect
        // namely the "onDisputeEnd multiple validators defending lost claim"
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        var lastValidator = validators[validators.length - 1];
        var secondLastValidator = validators[validators.length - 2];

        // all validators agree but the last two
        for (var i = 0; i < validators.length - 2; i++) {
            await debugFacet._onClaim(validators[i], claim);
        }

        // make all other validators but the last two defending the losing dispute
        for (var i = 0; i < validators.length - 3; i++) {
            await debugFacet._onDisputeEnd(
                lastValidator,
                validators[i],
                claim2
            );
        }
        // honest validator winning the last dispute
        // callStatic: check return value
        expect(
            JSON.stringify(
                await debugFacet.callStatic._onDisputeEnd(
                    lastValidator,
                    validators[validators.length - 3],
                    claim2
                )
            ),
            "use callStatic to check return value of onDisputeEnd() after last dispute"
        ).to.equal(
            JSON.stringify([
                Result.NoConflict,
                [claim2, hash_zero],
                [lastValidator, address_zero],
            ])
        );
        // check emitted event
        await expect(
            debugFacet._onDisputeEnd(
                lastValidator,
                validators[validators.length - 3],
                claim2
            ),
            "check emitted event for the last dispute"
        )
            .to.emit(debugFacet, "DisputeEnded")
            .withArgs(
                Result.NoConflict,
                [claim2, hash_zero],
                [lastValidator, address_zero]
            );

        // now the second last validator can finalize the consensus
        // callStatic: check return value
        expect(
            JSON.stringify(
                await debugFacet.callStatic._onClaim(
                    secondLastValidator,
                    claim2
                )
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
            debugFacet._onClaim(secondLastValidator, claim2),
            "finalize the consensus"
        )
            .to.emit(debugFacet, "ClaimReceived")
            .withArgs(
                Result.Consensus,
                [claim2, hash_zero],
                [secondLastValidator, address_zero]
            );
    });

    it("onNewEpoch", async () => {
        var claim = "0x" + "1".repeat(64);

        if (!enableDelegate) {
            // one validator claims
            await debugFacet._onClaim(validators[0], claim);

            // epoch ends without consensus
            // callStatic: check return value
            expect(
                await debugFacet.callStatic._onNewEpochVM(),
                "onNewEpoch() should return current claim"
            ).to.equal(claim);
            // check emitted event
            await expect(
                debugFacet._onNewEpochVM(),
                "new epoch should emit event NewEpoch with current claim"
            )
                .to.emit(debugFacet, "NewEpoch")
                .withArgs(claim);

            expect(
                await validatorManagerFacet.getAgreementMask(),
                "current agreement mask should reset"
            ).to.equal(0);

            expect(
                await validatorManagerFacet.getCurrentClaim(),
                "current claim should reset"
            ).to.equal(hash_zero);
        } else {
            // test delegate
            await passInputAccumulationPeriod();
            await rollupsFacet.connect(signers[0]).claim(claim);

            // only signers[0] claimed and new epoch begins
            // instead of using `debugFacet._onNewEpochVM();`
            // delegate needs to use `rollupsFacet.finalizeEpoch();`
            // so that Rollups will emit claim events with the correct epoch value
            await passChallengePeriod();
            await rollupsFacet.finalizeEpoch();
            await passInputAccumulationPeriod();

            let state = JSON.parse(await getState(initialState));

            // only signers[0] in `num_claims`
            expect(
                state.num_claims[0].validator_address,
                "only signers[0] in `num_claims`"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                state.num_claims[0].num_claims_made,
                "signers[0] made 1 claim"
            ).to.equal("0x1");
            for (let i = 1; i < 8; i++) {
                expect(
                    state.num_claims[i],
                    "all other fields in `num_claims` should be null"
                ).to.equal(null);
            }

            // new epoch
            expect(state.num_finalized_epochs, "enters epoch 1").to.equal(
                "0x1"
            );

            // no one has claimed in the new epoch yet
            expect(
                state.claiming.length,
                "no one has claimed in the new epoch yet"
            ).to.equal(0);

            // no one has lost a dispute yet
            expect(
                state.validators_removed.length,
                "no one has lost a dispute yet"
            ).to.equal(0);
        }
    });

    it("test #claims", async () => {
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);

        if (!enableDelegate) {
            // check initial #claims
            for (var i = 0; i < validators.length; i++) {
                expect(
                    await validatorManagerFacet.getNumberOfClaimsByAddress(
                        validators[i]
                    ),
                    "initial #claims"
                ).to.equal(0);

                expect(
                    await validatorManagerFacet.getNumberOfClaimsByIndex(i),
                    "initial #claims (for index)"
                ).to.equal(0);
            }

            // all validators make the same claim
            for (var i = 0; i < validators.length; i++) {
                await debugFacet._onClaim(validators[i], claim);
                expect(
                    await validatorManagerFacet.getNumberOfClaimsByAddress(
                        validators[i]
                    ),
                    "still 0 because claim hasn't finalized"
                ).to.equal(0);

                expect(
                    await validatorManagerFacet.getNumberOfClaimsByIndex(i),
                    "still 0 because claim hasn't finalized (for index)"
                ).to.equal(0);
            }
            // wait until claim finalized (either consensus or timeout)
            // new epoch begins and #claims increases
            await debugFacet._onNewEpochVM();
            for (var i = 0; i < validators.length; i++) {
                expect(
                    await validatorManagerFacet.getNumberOfClaimsByAddress(
                        validators[i]
                    ),
                    "now #claims increased"
                ).to.equal(1);

                expect(
                    await validatorManagerFacet.getNumberOfClaimsByIndex(i),
                    "now #claims increased (for index)"
                ).to.equal(1);
            }

            // keep skipping to new epoches
            for (let epoch = 1; epoch < 20; epoch++) {
                // 1st validator keeps making claims
                await debugFacet._onClaim(validators[0], claim);
                await debugFacet._onNewEpochVM();
                // check how #claims is increasing
                expect(
                    await validatorManagerFacet.getNumberOfClaimsByAddress(
                        validators[0]
                    ),
                    "check increasing #claims"
                ).to.equal(epoch + 1);

                expect(
                    await validatorManagerFacet.getNumberOfClaimsByIndex(0),
                    "check increasing #claims (for index)"
                ).to.equal(epoch + 1);
            }

            // #claims gets cleared once a validator makes a wrong claim
            await debugFacet._onClaim(validators[0], claim);
            await debugFacet._onClaim(validators[1], claim2);
            // let the 2nd validator win the dispute
            await debugFacet._onDisputeEnd(
                validators[1],
                validators[0],
                claim2
            );
            await debugFacet._onNewEpochVM();
            expect(
                await validatorManagerFacet.getNumberOfClaimsByAddress(
                    validators[0]
                ),
                "now the #claims for validator0 should get cleared"
            ).to.equal(0);
            expect(
                await validatorManagerFacet.getNumberOfClaimsByAddress(
                    validators[1]
                ),
                "#claims for validator1 should increase by 1"
            ).to.equal(2);

            // same for index methods
            expect(
                await validatorManagerFacet.getNumberOfClaimsByIndex(0),
                "now the #claims for validator0 should get cleared (for index)"
            ).to.equal(0);
            expect(
                await validatorManagerFacet.getNumberOfClaimsByIndex(1),
                "#claims for validator1 should increase by 1 (for index)"
            ).to.equal(2);
        } else {
            // test delegate
            await passInputAccumulationPeriod();
            for (let i = 0; i < 7; i++) {
                await rollupsFacet.connect(signers[i]).claim(claim);
            }
            let state = JSON.parse(await getState(initialState));
            for (let i = 0; i < 8; i++) {
                expect(
                    state.num_claims[i],
                    "no one has #claims increased yet, because epoch hasn't finalized"
                ).to.equal(null);
            }

            // only until the epoch finalized, will their #claims increase
            // instead of using `debugFacet._onNewEpochVM();`
            // delegate needs to use `rollupsFacet.finalizeEpoch();`
            // so that Rollups will emit claim events with the correct epoch value
            await passChallengePeriod();
            await rollupsFacet.finalizeEpoch();
            await passInputAccumulationPeriod();
            state = JSON.parse(await getState(initialState));

            for (let i = 0; i < 7; i++) {
                expect(
                    state.num_claims[i].validator_address,
                    "now validators claimed has #claims increased"
                ).to.equal((await signers[i].getAddress()).toLowerCase());
                expect(
                    state.num_claims[i].num_claims_made,
                    "now validators claimed has 1 claim"
                ).to.equal("0x1");
            }
            expect(
                state.num_claims[7],
                "the last validator didn't make a claim"
            ).to.equal(null);

            // keep skipping to new epoches
            for (let epoch = 1; epoch < 20; epoch++) {
                // 1st validator keeps making claims
                await rollupsFacet.connect(signers[0]).claim(claim);

                // signers[0] should be in the `claiming`
                state = JSON.parse(await getState(initialState));
                expect(
                    state.claiming[0],
                    "signers[0] should be in the `claiming`"
                ).to.equal((await signers[0].getAddress()).toLowerCase());
                // and its #claims should stay the same, until epoch finalized
                expect(
                    parseInt(state.num_claims[0].num_claims_made, 16),
                    "#claims should stay the same for now"
                ).to.equal(epoch);

                // finalize epoch
                // instead of using `debugFacet._onNewEpochVM();`
                // delegate needs to use `rollupsFacet.finalizeEpoch();`
                // so that Rollups will emit claim events with the correct epoch value
                await passChallengePeriod();
                await rollupsFacet.finalizeEpoch();
                await passInputAccumulationPeriod();

                state = JSON.parse(await getState(initialState));
                // check how #claims is increasing
                expect(
                    state.num_claims[0].validator_address,
                    "the 1st in `num_claims` is the 1st validator"
                ).to.equal((await signers[0].getAddress()).toLowerCase());
                expect(
                    parseInt(state.num_claims[0].num_claims_made, 16),
                    "the 1st validator keeps increasing #claims"
                ).to.equal(epoch + 1);
            }

            // once a valiator lost a dispute, it will get removed
            // and #claims will be cleared
            await rollupsFacet.connect(signers[0]).claim(claim);
            await rollupsFacet.connect(signers[1]).claim(claim2);
            state = JSON.parse(await getState(initialState));

            // now signers[1] will be removed from `num_claims`
            // and added into `validators_removed`
            expect(
                state.num_claims[1],
                "the 2nd field of `num_claims` should be null"
            ).to.equal(null);
            expect(
                state.validators_removed[0],
                "should be added into `validators_removed`"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(state.validators_removed.length, "only 1 removed").to.equal(
                1
            );

            // check all other values
            expect(
                state.num_claims[0].validator_address,
                "the first address in `num_claims`"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                state.num_claims[0].num_claims_made,
                "the 1st validator should have 20 claims, the last claim hasn't finalized"
            ).to.equal("0x14");
            for (let i = 2; i < 8 - 1; i++) {
                expect(
                    state.num_claims[i].validator_address,
                    "the i-th address in `num_claims`"
                ).to.equal((await signers[i].getAddress()).toLowerCase());
                expect(
                    state.num_claims[i].num_claims_made,
                    "they all have only 1 claim"
                ).to.equal("0x1");
            }
            expect(
                state.num_claims[7],
                "the last validator hasn't made a claim yet"
            ).to.equal(null);
            // the last epoch hasn't finalized yet
            // and signers[0] has claimed
            expect(
                state.claiming.length,
                "no one has claimed in the current epoch"
            ).to.equal(1);
            expect(
                state.claiming[0],
                "signers[0] has claimed in the current epoch"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            // there are 20 finalized epochs
            expect(state.num_finalized_epochs, "20 finalized epochs").to.equal(
                "0x14"
            );
        }
    });

    it("test getValidatorIndex() and its revert behavior", async () => {
        for (let i = 0; i < 8; i++) {
            expect(
                await validatorManagerFacet.getValidatorIndex(validators[i]),
                "check the return value of getValidatorIndex()"
            ).to.equal(i);
        }

        // now test for an outsider
        await expect(
            validatorManagerFacet.getValidatorIndex(address_zero),
            "address 0, should revert"
        ).to.be.revertedWith("address 0");
        await expect(
            validatorManagerFacet.getValidatorIndex(address_one),
            "address not in the validator set"
        ).to.be.revertedWith("validator not found");

        // now test when some validator gets kicked out
        var claim = "0x" + "1".repeat(64);
        var claim2 = "0x" + "2".repeat(64);
        await debugFacet._onClaim(validators[0], claim);
        await debugFacet._onClaim(validators[1], claim2);
        // let the 2nd validator lose the dispute
        await debugFacet._onDisputeEnd(validators[0], validators[1], claim);
        await expect(
            validatorManagerFacet.getValidatorIndex(validators[1]),
            "validators[1] gets kicked out, should revert"
        ).to.be.revertedWith("validator not found");
        for (let i = 0; i < 8 && i != 1; i++) {
            expect(
                await validatorManagerFacet.getValidatorIndex(validators[i]),
                "other validators should still work fine"
            ).to.equal(i);
        }
    });
});
