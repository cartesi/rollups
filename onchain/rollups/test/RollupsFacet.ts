import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { RollupsFacet } from "../src/types/RollupsFacet";
import { RollupsFacet__factory } from "../src/types/factories/RollupsFacet__factory";
import { DiamondInit } from "../src/types/DiamondInit";
import { DiamondInit__factory } from "../src/types/factories/DiamondInit__factory";
import { deployDiamond, getState } from "./utils";

use(solidity);

describe("Rollups Facet", () => {
    /// for testing Rollups when modifiers are on, set this to true
    /// for testing Rollups when modifiers are off, set this to false
    let permissionModifiersOn = true;

    let enableDelegate = process.env["DELEGATE_TEST"];

    let rollupsFacet: RollupsFacet;
    let diamondInit: DiamondInit;

    const MINUTE = 60; // seconds in a minute
    const HOUR = 60 * MINUTE; // seconds in an hour
    const DAY = 24 * HOUR; // seconds in a day

    const inputDuration = 1 * DAY;
    const challengePeriod = 7 * DAY;
    const INPUT_LOG2_SIZE = 25;
    const VOUCHER_METADATA_LOG2_SIZE = 21;

    let signers: Signer[];

    ///each address is 20 bytes
    let address_zero = "0x0000000000000000000000000000000000000000";

    let numberOfFinalizedEpochs = 10;

    ///let enum starts from 0
    enum Phase {
        InputAccumulation = 0,
        AwaitingConsensus = 1,
        AwaitingDispute = 2,
    }
    enum Result {
        NoConflict = 0,
        Consensus = 1,
        Conflict = 2,
    }

    // creation timestamp for rollups
    let contract_creation_time: any;

    // initial var for delegate
    let initialEpoch: any;
    let initialState: any;

    beforeEach(async () => {
        signers = await ethers.getSigners();

        let validators: string[] = [];

        for (let i = 0; i < 3; i++) {
            let address = await signers[i].getAddress();
            validators.push(address);
        }

        const diamond = await deployDiamond({
            inputLog2Size: INPUT_LOG2_SIZE,
            validators: validators,
        });
        rollupsFacet = RollupsFacet__factory.connect(
            diamond.address,
            signers[0]
        );
        diamondInit = DiamondInit__factory.connect(diamond.address, signers[0]);
        // get the timestamp of the second last block, because after deploying rollups, portalImpl was deployed
        contract_creation_time =
            (await ethers.provider.getBlock("latest")).timestamp - 1;

        initialEpoch = "0x0";
        initialState = JSON.stringify({
            initial_epoch: initialEpoch,
            rollups_address: diamond.address,
        });
    });

    /// ***test public variable currentPhase*** ///
    it("initial phase should be InputAccumulation", async () => {
        expect(
            await rollupsFacet.getCurrentPhase(),
            "initial phase check"
        ).to.equal(Phase.InputAccumulation);
    });

    /// ***test function claim()*** ///
    it("calling claim() should revert if input duration has not yet past", async () => {
        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello")),
            "phase incorrect because inputDuration not over"
        ).to.be.revertedWith("Phase != AwaitingConsensus");

        await network.provider.send("evm_increaseTime", [inputDuration / 2]);
        await network.provider.send("evm_mine");

        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello")),
            "phase incorrect because inputDuration not over"
        ).to.be.revertedWith("Phase != AwaitingConsensus");
    });

    it("should claim() and enter into AwaitingConsensus phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));
        expect(
            await rollupsFacet.getCurrentPhase(),
            "current phase should be updated to AwaitingConsensus"
        ).to.equal(Phase.AwaitingConsensus);
    });

    it("should claim() and enter into InputAccumulation phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        // all validators agree with claim
        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));
        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("hello"));

        expect(
            await rollupsFacet.getCurrentPhase(),
            "current phase should be updated to InputAccumulation"
        ).to.equal(Phase.InputAccumulation);
    });

    it("conflicting claims by validators should end in AwaitingConsensus phase if not all validators claimed", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));
        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("not hello"));

        // In this version disputes get solved immediately
        // so the phase should be awaiting consensus after a disagreement
        expect(
            await rollupsFacet.getCurrentPhase(),
            "current phase should be updated to AwaitingConsensus"
        ).to.equal(Phase.AwaitingConsensus);
    });

    it("conflicting claims by validators should end in InputAccumulation, if all other validators had claimed beforehand", async () => {
        ///make two different claims///
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("not hello"));
        ///END: make two different claims///

        expect(
            await rollupsFacet.getCurrentPhase(),
            "current phase should be updated to InputAccumulation"
        ).to.equal(Phase.InputAccumulation);
    });

    /// ***test function finalizeEpoch()*** ///
    it("finalizeEpoch(): should revert if currentPhase is InputAccumulation", async () => {
        await expect(
            rollupsFacet.finalizeEpoch(),
            "phase incorrect"
        ).to.be.revertedWith("Phase != Awaiting Consensus");
    });

    // The phase is never AwaitingDispute in the end of at transaction, in this version
    //it("finalizeEpoch(): should revert if currentPhase is AwaitingDispute", async () => {
    //    ///make two different claims///
    //    await network.provider.send("evm_increaseTime", [inputDuration + 1]);
    //    await network.provider.send("evm_mine");

    //    await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

    //    await rollupsFacet
    //        .connect(signers[1])
    //        .claim(ethers.utils.formatBytes32String("halo"));
    //    ///END: make two different claims///

    //    await expect(
    //        rollupsFacet.finalizeEpoch(),
    //        "phase incorrect"
    //    ).to.be.revertedWith("Phase != Awaiting Consensus");
    //});

    it("finalizeEpoch(): should revert if challengePeriod is not over", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            rollupsFacet.finalizeEpoch(),
            "Challenge period is not over"
        ).to.be.revertedWith("Challenge period not over");
    });

    it("claim(): should revert if the current claim is null", async () => {
        let currentClaim = ethers.utils.formatBytes32String("\0");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await expect(
            rollupsFacet.claim(currentClaim),
            "empty claim"
        ).to.be.revertedWith("empty claim");
    });

    it("after finalizeEpoch(), current phase should be InputAccumulation", async () => {
        let currentClaim = ethers.utils.formatBytes32String("hello");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(currentClaim);

        await network.provider.send("evm_increaseTime", [challengePeriod + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.finalizeEpoch();

        expect(
            await rollupsFacet.getCurrentPhase(),
            "final phase check"
        ).to.equal(Phase.InputAccumulation);
    });

    /// ***test emitting events*** ///
    it("event RollupsInitialized", async () => {
        // we use ethers.js to query historic events
        // ref: https://docs.ethers.io/v5/single-page/#/v5/getting-started/-%23-getting-started--history
        let eventFilter = diamondInit.filters.RollupsInitialized(null, null);
        let event = await diamondInit.queryFilter(eventFilter);
        let eventArgs = event[0]["args"]; // get 'args' from the first RollupsInitialized event

        expect(eventArgs["inputDuration"], "Input Duration").to.equal(
            inputDuration
        );

        expect(eventArgs["challengePeriod"], "Challenge Period").to.equal(
            challengePeriod
        );
    });

    it("event Claim for NoConflict and Conflict", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                0,
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        await expect(
            rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("not hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                0,
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("not hello")
            );
    });

    it("event Claim for Consensus", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                0,
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        // skip input duration
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        // claim epoch 1
        await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                1,
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
    });

    it("event PhaseChange", async () => {
        //advance input duration from input accumulation start
        await network.provider.send("evm_increaseTime", [
            (await rollupsFacet.getInputAccumulationStart()).toNumber() +
                inputDuration +
                1,
        ]);
        await network.provider.send("evm_mine");

        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "PhaseChange")
            .withArgs(Phase.AwaitingConsensus);

        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        //event PhaseChange: InputAccumulation
        await expect(
            rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "PhaseChange")
            .withArgs(Phase.InputAccumulation);

        // @dev this version doesnt include Awaiting Dispute phase
        //event PhaseChange: AwaitingDispute
        //await expect(
        //    rollupsFacet
        //        .connect(signers[1])
        //        .claim(ethers.utils.formatBytes32String("halo"))
        //)
        //    .to.emit(rollupsFacet, "PhaseChange")
        //    .withArgs(Phase.AwaitingDispute);
    });

    it("event FinalizeEpoch", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await rollupsFacet
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("hello"));

        await rollupsFacet
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "FinalizeEpoch")
            .withArgs(0, ethers.utils.formatBytes32String("hello"));
    });

    it("getCurrentEpoch() without conflict", async () => {
        // initial epoch number
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(0);

        let epochNum = 0;

        // epoch number increases when input accumulation finishes
        // the length of finalized epochs array increases upon consensus without conflict
        for (let i = 0; i < 9; i++) {
            // input accumulation
            expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

            // input accumulation over
            // ***epoch increases by 1***
            // but output.getNumberOfFinalizedEpochs() stays the same temporarily
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");
            epochNum++;

            await expect(
                rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
            )
                .to.emit(rollupsFacet, "Claim")
                .withArgs(
                    epochNum - 1, // claim for the previous epoch
                    await signers[0].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                );

            await rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("hello"));

            expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

            await expect(
                rollupsFacet
                    .connect(signers[2])
                    .claim(ethers.utils.formatBytes32String("hello"))
            )
                .to.emit(rollupsFacet, "Claim")
                .withArgs(
                    epochNum - 1, // claim for the previous epoch
                    await signers[2].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                );
            // enter input accumulation again
            // ***the length of finalized epochs array increases by 1***
            // now it is the same as the epoch number
            expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);
        }
    });

    it("getCurrentEpoch() with conflict", async () => {
        // initial epoch number
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(0);

        let epochNum = 0;

        // input accumulation
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

        // input accumulation over
        // ***epoch increases by 1***
        // but output.getNumberOfFinalizedEpochs() stays the same temporarily
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");
        epochNum++;

        // first claim
        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

        // 2nd claim => conflict
        await expect(
            rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("halo"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("halo")
            );

        // 3rd claim => Consensus
        await expect(
            rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        // enter input accumulation again
        // ***the length of finalized epochs array increases by 1***
        // now it is the same as the epoch number
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

        // in this epoch, signers[1] is already deleted
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");
        epochNum++;
        // first claim
        await expect(
            rollupsFacet.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);

        // 2nd claim => revert because claimer lost the dispute before
        await expect(
            rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("hello"))
        ).to.be.revertedWith("sender not allowed");

        // 3rd claim => Consensus
        await expect(
            rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(rollupsFacet, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        // enter input accumulation again
        expect(await rollupsFacet.getCurrentEpoch()).to.equal(epochNum);
    });

    // test delegate
    if (enableDelegate) {
        /* example Rollups delegate output looks like 
        {
            constants: {
                input_duration: '0x15180',
                challenge_period: '0x93a80',
                contract_creation_timestamp: '0x616e3ac3',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81',
                output_contract_address: '0x6d544390eb535d61e196c87d6b9c80dcd8628acd',
                validator_contract_address: '0xb1ede3f5ac8654124cb5124adf0fd3885cbdd1f7',
                dispute_contract_address: '0xa6d6d7c556ce6ada136ba32dbe530993f128ca44',
                rollups_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9'
            },
            initial_epoch: '0x0',
            finalized_epochs: {
                finalized_epochs: [],
                initial_epoch: '0x0',
                rollups_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
            },
            current_epoch: {
                epoch_number: '0x0',
                inputs: {
                epoch_number: '0x0',
                inputs: [],
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
                },
                rollups_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
            },
            current_phase: { InputAccumulation: {} },
            output_state: {
                output_address: '0x6d544390eb535d61e196c87d6b9c80dcd8628acd',
                vouchers: {}
            }
        }
        */

        it("test delegate", async () => {
            let state = JSON.parse(await getState(initialState));

            // *** initial test ***

            // test constants
            expect(
                parseInt(state.constants.input_duration, 16),
                "input duration does not match"
            ).to.equal(inputDuration);
            expect(
                parseInt(state.constants.challenge_period, 16),
                "challenge period does not match"
            ).to.equal(challengePeriod);
            expect(
                parseInt(state.constants.contract_creation_timestamp, 16),
                "contract creation timestamp does not match"
            ).to.equal(contract_creation_time);
            expect(
                state.constants.rollups_contract_address,
                "rollups contract address does not match"
            ).to.equal(rollupsFacet.address.toLowerCase());

            // test initial_epoch
            expect(
                state.initial_epoch,
                "initial epoch does not match"
            ).to.equal(initialEpoch);

            // test initial finalized_epochs
            expect(
                state.finalized_epochs.finalized_epochs.length,
                "initial finalized_epochs.finalized_epochs does not match"
            ).to.equal(0);
            expect(
                state.finalized_epochs.initial_epoch,
                "finalized_epochs.initial_epoch does not match"
            ).to.equal(initialEpoch);
            expect(
                state.finalized_epochs.rollups_contract_address,
                "finalized_epochs.rollups_contract_address does not match"
            ).to.equal(rollupsFacet.address.toLowerCase());

            // test initial current_epoch
            checkCurrentEpochNum(state, initialEpoch);
            expect(
                state.current_epoch.inputs.epoch_number,
                "initial current_epoch.inputs.epoch_number does not match"
            ).to.equal(initialEpoch);
            expect(
                state.current_epoch.inputs.inputs.length,
                "initially there's no inputs"
            ).to.equal(0);
            expect(
                state.current_epoch.rollups_contract_address,
                "current_epoch.rollups_contract_address does not match"
            ).to.equal(rollupsFacet.address.toLowerCase());
            expect(
                JSON.stringify(state.current_phase.InputAccumulation) == "{}",
                "initial phase"
            ).to.equal(true);
            expect(
                JSON.stringify(state.output_state.vouchers) == "{}",
                "initially there's no vouchers"
            ).to.equal(true);

            // *** EPOCH 0: claim when the input duration has not past ***
            await expect(
                rollupsFacet.claim(ethers.utils.formatBytes32String("hello")),
                "phase incorrect because inputDuration not over"
            ).to.be.revertedWith("Phase != AwaitingConsensus");
            await network.provider.send("evm_increaseTime", [
                inputDuration / 2,
            ]);
            await network.provider.send("evm_mine");
            await expect(
                rollupsFacet.claim(ethers.utils.formatBytes32String("hello")),
                "phase incorrect because inputDuration not over"
            ).to.be.revertedWith("Phase != AwaitingConsensus");

            state = JSON.parse(await getState(initialState)); // update state
            checkCurrentPhase(state, "InputAccumulation");

            // *** EPOCH 0: input duration has past, now make a claim ***
            await network.provider.send("evm_increaseTime", [
                inputDuration / 2 + 1,
            ]);
            await network.provider.send("evm_mine");
            await rollupsFacet.claim(ethers.utils.formatBytes32String("hello"));

            state = JSON.parse(await getState(initialState)); // update state
            checkCurrentEpochNum(state, "0x1");
            checkCurrentPhase(state, "AwaitingConsensusNoConflict");
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusNoConflict
                        .claimed_epoch.epoch_number,
                    16
                ),
                "claimed epoch"
            ).to.equal(0);
            expect(
                ethers.utils.formatBytes32String("hello") in
                    state.current_phase.AwaitingConsensusNoConflict
                        .claimed_epoch.claims.claims,
                "the value of the claim does not match"
            ).to.equal(true);
            expect(
                state.current_phase.AwaitingConsensusNoConflict.claimed_epoch
                    .claims.claims[
                    ethers.utils.formatBytes32String("hello")
                ][0],
                "the sender address of the claim does not match"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusNoConflict
                        .claimed_epoch.claims.first_claim_timestamp,
                    16
                ),
                "the timestamp of the first claim does not match"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);
            // inputs are tested in the input delegate tests

            // *** EPOCH 0: claim to reach consensus ***
            await rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("hello"));

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                state.current_phase.AwaitingConsensusNoConflict.claimed_epoch
                    .claims.claims[ethers.utils.formatBytes32String("hello")]
                    .length,
                "now there are 2 claimers having the same claim"
            ).to.equal(2);
            expect(
                state.current_phase.AwaitingConsensusNoConflict.claimed_epoch.claims.claims[
                    ethers.utils.formatBytes32String("hello")
                ].includes((await signers[0].getAddress()).toLowerCase()),
                "signers[0] should be in the claimers list"
            ).to.equal(true);
            expect(
                state.current_phase.AwaitingConsensusNoConflict.claimed_epoch.claims.claims[
                    ethers.utils.formatBytes32String("hello")
                ].includes((await signers[1].getAddress()).toLowerCase()),
                "signers[1] should be in the claimers list"
            ).to.equal(true);

            await rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"));

            state = JSON.parse(await getState(initialState)); // update state
            await checkFinalizedEpoch(
                state,
                0,
                ethers.utils.formatBytes32String("hello")
            );
            checkCurrentEpochNum(state, "0x1");
            checkCurrentPhase(state, "InputAccumulation");

            // *** EPOCH 1: sealed epoch ***
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            state = JSON.parse(await getState(initialState)); // update state
            checkCurrentEpochNum(state, "0x2");
            checkCurrentPhase(state, "EpochSealedAwaitingFirstClaim");
            expect(
                state.current_phase.EpochSealedAwaitingFirstClaim.sealed_epoch
                    .epoch_number,
                "the sealed epoch number does not match"
            ).to.equal("0x1");

            // *** EPOCH 1: conflicting claims ***
            await rollupsFacet.claim(
                ethers.utils.formatBytes32String("hello1")
            );
            let first_claim_timestamp = (
                await ethers.provider.getBlock("latest")
            ).timestamp;
            await rollupsFacet
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("not hello1"));

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                state.current_phase.AwaitingConsensusAfterConflict.claimed_epoch
                    .epoch_number,
                "claims are for epoch 1"
            ).to.equal("0x1");
            expect(
                state.current_phase.AwaitingConsensusAfterConflict.claimed_epoch
                    .claims.claims[
                    ethers.utils.formatBytes32String("hello1")
                ][0],
                "address of the first claim does not match"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                state.current_phase.AwaitingConsensusAfterConflict.claimed_epoch
                    .claims.claims[
                    ethers.utils.formatBytes32String("not hello1")
                ][0],
                "address of the challenging claim does not match"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusAfterConflict
                        .claimed_epoch.claims.first_claim_timestamp,
                    16
                ),
                "timestamp of the first claim does not match"
            ).to.equal(first_claim_timestamp);
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusAfterConflict
                        .challenge_period_base_ts,
                    16
                ),
                "timestamp of the challenging claim does not match"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);

            // *** EPOCH 1: consensus waiting period times out ***
            await network.provider.send("evm_increaseTime", [
                challengePeriod + 1,
            ]);
            await network.provider.send("evm_mine");

            state = JSON.parse(await getState(initialState)); // update state
            checkCurrentPhase(state, "ConsensusTimeout");
            expect(
                state.current_phase.ConsensusTimeout.claimed_epoch.epoch_number,
                "epoch number when ConsensusTimeout"
            ).to.equal("0x1");

            // *** EPOCH 1 -> 2: finalize after consensus times out ***
            await rollupsFacet.finalizeEpoch();

            state = JSON.parse(await getState(initialState)); // update state
            // now can test the finalized epoch 1
            await checkFinalizedEpoch(
                state,
                1,
                ethers.utils.formatBytes32String("hello1")
            );

            checkCurrentPhase(state, "InputAccumulation");

            // *** EPOCH 2 -> 3: conflicting claims but reach consensus once conflict is resolved ***
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await rollupsFacet.claim(
                ethers.utils.formatBytes32String("hello2")
            );
            await rollupsFacet
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("not hello2"));

            state = JSON.parse(await getState(initialState)); // update state
            checkCurrentEpochNum(state, "0x3");
            checkCurrentPhase(state, "InputAccumulation");
            await checkFinalizedEpoch(
                state,
                2,
                ethers.utils.formatBytes32String("hello2")
            );
        });
    }
});

function checkCurrentPhase(state: any, phase: string) {
    expect(
        phase in state.current_phase,
        "current phase does not match"
    ).to.equal(true);
}

function checkCurrentEpochNum(state: any, epoch: string) {
    expect(
        state.current_epoch.epoch_number,
        "current epoch number does not match"
    ).to.equal(epoch);
}

// should await for this function
async function checkFinalizedEpoch(
    state: any,
    epoch: number,
    epochHash: string
) {
    expect(
        parseInt(
            state.finalized_epochs.finalized_epochs[epoch].epoch_number,
            16
        ),
        "finalized epoch number does not match"
    ).to.equal(epoch);
    expect(
        state.finalized_epochs.finalized_epochs[epoch].hash,
        "finalized hash does not match"
    ).to.equal(epochHash);
    expect(
        state.finalized_epochs.finalized_epochs[epoch].finalized_block_hash,
        "finalized_block_hash does not match"
    ).to.equal((await ethers.provider.getBlock("latest")).hash);
    expect(
        parseInt(
            state.finalized_epochs.finalized_epochs[epoch]
                .finalized_block_number,
            16
        ),
        "finalized_block_number does not match"
    ).to.equal((await ethers.provider.getBlock("latest")).number);
}
