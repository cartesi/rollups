import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { DescartesV2Impl } from "../src/types/DescartesV2Impl";
import { DescartesV2Impl__factory } from "../src/types/factories/DescartesV2Impl__factory";
import { getState } from "./getState";

use(solidity);

describe("Descartes V2 Implementation", () => {
    /// for testing DescartesV2 when modifiers are on, set this to true
    /// for testing DescartesV2 when modifiers are off, set this to false
    let permissionModifiersOn = true;
    let runWithDeployScript = true;

    let enableDelegate = process.env["DELEGATE_TEST"];

    let descartesV2Impl: DescartesV2Impl;

    const MINUTE = 60; // seconds in a minute
    const HOUR = 60 * MINUTE; // seconds in an hour
    const DAY = 24 * HOUR; // seconds in a day

    const inputDuration = 1 * DAY;
    const challengePeriod = 7 * DAY;
    const INPUT_LOG2_SIZE = 25;
    const OUTPUT_METADATA_LOG2_SIZE = 21;

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

    // creation timestamp for descartesV2
    let contract_creation_time: any;

    // initial var for delegate
    let initialEpoch: any;
    let initialState: any;

    beforeEach(async () => {
        signers = await ethers.getSigners();

        if (runWithDeployScript) {
            await deployments.fixture();
            const dAddress = (await deployments.get("DescartesV2Impl")).address;
            descartesV2Impl = DescartesV2Impl__factory.connect(
                dAddress,
                signers[0]
            );
            // get the timestamp of the second last block, because after deploying descartesV2, portalImpl was deployed
            contract_creation_time =
                (await ethers.provider.getBlock("latest")).timestamp - 1;
        } else {
            // Bitmask
            const bitMaskLibrary = await deployments.deploy("Bitmask", {
                from: await signers[0].getAddress(),
            });
            const bitMaskAddress = bitMaskLibrary.address;

            // CartesiMath
            const cartesiMath = await deployments.deploy("CartesiMath", {
                from: await signers[0].getAddress(),
            });
            const cartesiMathAddress = cartesiMath.address;

            // Merkle
            const merkle = await deployments.deploy("Merkle", {
                from: await signers[0].getAddress(),
                libraries: {
                    CartesiMath: cartesiMathAddress,
                },
            });
            const merkleAddress = merkle.address;

            // DescartesV2Impl
            const descartesV2Impl_factory = await ethers.getContractFactory(
                "DescartesV2Impl",
                {
                    signer: signers[0],
                    libraries: {
                        Bitmask: bitMaskAddress,
                        Merkle: merkleAddress,
                    },
                }
            );
            descartesV2Impl = await descartesV2Impl_factory.deploy(
                inputDuration,
                challengePeriod,
                INPUT_LOG2_SIZE,
                OUTPUT_METADATA_LOG2_SIZE,
                [
                    await signers[0].getAddress(),
                    await signers[1].getAddress(),
                    await signers[2].getAddress(),
                ]
            );
            await descartesV2Impl.deployed();
            contract_creation_time = (await ethers.provider.getBlock("latest"))
                .timestamp;
        }

        initialEpoch = "0x0";
        initialState = JSON.stringify({
            initial_epoch: initialEpoch,
            descartes_address: descartesV2Impl.address,
        });
    });

    /// ***test public variable currentPhase*** ///
    it("initial phase should be InputAccumulation", async () => {
        expect(
            await descartesV2Impl.getCurrentPhase(),
            "initial phase check"
        ).to.equal(Phase.InputAccumulation);
    });

    /// ***test function claim()*** ///
    it("calling claim() should revert if input duration has not yet past", async () => {
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello")),
            "phase incorrect because inputDuration not over"
        ).to.be.revertedWith("Phase != AwaitingConsensus");

        await network.provider.send("evm_increaseTime", [inputDuration / 2]);
        await network.provider.send("evm_mine");

        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello")),
            "phase incorrect because inputDuration not over"
        ).to.be.revertedWith("Phase != AwaitingConsensus");
    });

    it("should claim() and enter into AwaitingConsensus phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        expect(
            await descartesV2Impl.getCurrentPhase(),
            "current phase should be updated to AwaitingConsensus"
        ).to.equal(Phase.AwaitingConsensus);
    });

    it("should claim() and enter into InputAccumulation phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        // all validators agree with claim
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("hello"));

        expect(
            await descartesV2Impl.getCurrentPhase(),
            "current phase should be updated to InputAccumulation"
        ).to.equal(Phase.InputAccumulation);
    });

    it("conflicting claims by validators should end in AwaitingConsensus phase if not all validators claimed", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("not hello"));

        // In this version disputes get solved immediately
        // so the phase should be awaiting consensus after a disagreement
        expect(
            await descartesV2Impl.getCurrentPhase(),
            "current phase should be updated to AwaitingConsensus"
        ).to.equal(Phase.AwaitingConsensus);
    });

    it("conflicting claims by validators should end in InputAccumulation, if all other validators had claimed beforehand", async () => {
        ///make two different claims///
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("not hello"));
        ///END: make two different claims///

        expect(
            await descartesV2Impl.getCurrentPhase(),
            "current phase should be updated to InputAccumulation"
        ).to.equal(Phase.InputAccumulation);
    });

    /// ***test function finalizeEpoch()*** ///
    it("finalizeEpoch(): should revert if currentPhase is InputAccumulation", async () => {
        await expect(
            descartesV2Impl.finalizeEpoch(),
            "phase incorrect"
        ).to.be.revertedWith("Phase != Awaiting Consensus");
    });

    // The phase is never AwaitingDispute in the end of at transaction, in this version
    //it("finalizeEpoch(): should revert if currentPhase is AwaitingDispute", async () => {
    //    ///make two different claims///
    //    await network.provider.send("evm_increaseTime", [inputDuration + 1]);
    //    await network.provider.send("evm_mine");

    //    await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

    //    await descartesV2Impl
    //        .connect(signers[1])
    //        .claim(ethers.utils.formatBytes32String("halo"));
    //    ///END: make two different claims///

    //    await expect(
    //        descartesV2Impl.finalizeEpoch(),
    //        "phase incorrect"
    //    ).to.be.revertedWith("Phase != Awaiting Consensus");
    //});

    it("finalizeEpoch(): should revert if challengePeriod is not over", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            descartesV2Impl.finalizeEpoch(),
            "Challenge period is not over"
        ).to.be.revertedWith("Challenge period not over");
    });

    it("claim(): should revert if the current claim is null", async () => {
        let currentClaim = ethers.utils.formatBytes32String("\0");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await expect(
            descartesV2Impl.claim(currentClaim),
            "empty claim"
        ).to.be.revertedWith("empty claim");
    });

    it("after finalizeEpoch(), current phase should be InputAccumulation", async () => {
        let currentClaim = ethers.utils.formatBytes32String("hello");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(currentClaim);

        await network.provider.send("evm_increaseTime", [challengePeriod + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.finalizeEpoch();

        expect(
            await descartesV2Impl.getCurrentPhase(),
            "final phase check"
        ).to.equal(Phase.InputAccumulation);
    });

    /// modifiers on
    if (permissionModifiersOn) {
        /// ***test function notifyInput() with modifier*** ///
        it("only input contract can call notifyInput()", async () => {
            await expect(
                descartesV2Impl.notifyInput(),
                "msg.sender != input contract"
            ).to.be.revertedWith("only Input Contract");
        });

        /// ***test function resolveDispute() with modifier*** ///
        it("only DisputeManager contract can call resolveDispute()", async () => {
            await expect(
                descartesV2Impl.resolveDispute(
                    await signers[0].getAddress(),
                    await signers[1].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                ),
                "msg.sender != dispute manager contract"
            ).to.be.revertedWith("only Dispute Contract");
        });
    }

    /// modifiers off
    if (!permissionModifiersOn) {
        /// ***test function notifyInput() without modifier*** ///
        it("notifyInput(): should return false if inputDuration has not past yet", async () => {
            expect(
                //callStatic is from ethers.js, used to call a state changing function without acutally changing the states. Use callStatic in order to check the return value.
                await descartesV2Impl.callStatic.notifyInput(),
                "inputDuration has not past yet"
            ).to.equal(false);
        });

        it("notifyInput(): if inputDuration has past and current phase is InputAccumulation, should return true and update the current phase to AwaitingConsensus", async () => {
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            expect(
                await descartesV2Impl.callStatic.notifyInput(),
                "should return true"
            ).to.equal(true);

            await descartesV2Impl.notifyInput(); //actually change states before getting current phase
            expect(
                await descartesV2Impl.getCurrentPhase(),
                "the updated current phase"
            ).to.equal(Phase.AwaitingConsensus);
        });

        it("notifyInput(): should return false if currentPhase is AwaitingDispute", async () => {
            ///make two different claims///
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await descartesV2Impl.claim(
                ethers.utils.formatBytes32String("hello")
            );

            await descartesV2Impl
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("halo"));
            ///END: make two different claims///

            expect(
                await descartesV2Impl.callStatic.notifyInput(),
                "phase incorrect"
            ).to.equal(false);
        });

        it("notifyInput(): if called more than once in an epoch, return false.", async () => {
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await descartesV2Impl.notifyInput();
            //Now the current phase is AwaitingConsensus
            expect(
                await descartesV2Impl.callStatic.notifyInput(),
                "repeated calling"
            ).to.equal(false);
        });

        // the following 3 tests are unnecessary because they are already tested with claim() function during conflicts
        /// ***test function resolveDispute() without modifier*** ///
        // it("resolveDispute(): if consensus, updated current phase should be InputAccumulation", async () => {
        //     await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        //     await descartesV2Impl.connect(signers[2]).claim(ethers.utils.formatBytes32String("hello"));
        //     await descartesV2Impl.resolveDispute(
        //         await signers[0].getAddress(),
        //         await signers[1].getAddress(),
        //         ethers.utils.formatBytes32String("hello")
        //     );
        //     expect(
        //         await descartesV2Impl.getCurrentPhase(),
        //         "updated current phase if consensus"
        //     ).to.equal(Phase.InputAccumulation);
        // });

        // it("resolveDispute(): if NoConflict, updated current phase should be AwaitingConsensus", async () => {
        //     await descartesV2Impl.resolveDispute(
        //         await signers[0].getAddress(),
        //         await signers[1].getAddress(),
        //         ethers.utils.formatBytes32String("hello")
        //     );
        //     console.log(await descartesV2Impl.getCurrentPhase())
        //     expect(
        //         await descartesV2Impl.getCurrentPhase(),
        //         "updated current phase if consensus"
        //     ).to.equal(Phase.AwaitingConsensus);
        // });

        // it("resolveDispute(): if Conflict, updated current phase should be AwaitingDispute", async () => {
        //     await descartesV2Impl.resolveDispute(
        //         await signers[0].getAddress(),
        //         await signers[1].getAddress(),
        //         ethers.utils.formatBytes32String("hello")
        //     );

        //     expect(
        //         await descartesV2Impl.getCurrentPhase(),
        //         "updated current phase if Conflict"
        //     ).to.equal(Phase.AwaitingDispute);
        //     //then start new dispute all over again
        // });
    }

    /// ***test emitting events*** ///
    it("event DescartesV2Created", async () => {
        // we use ethers.js to query historic events
        // ref: https://docs.ethers.io/v5/single-page/#/v5/getting-started/-%23-getting-started--history
        let eventFilter = descartesV2Impl.filters.DescartesV2Created(
            null,
            null,
            null,
            null,
            null,
            null
        );
        let event = await descartesV2Impl.queryFilter(eventFilter);
        let eventArgs = event[0]["args"]; // get 'args' from the first DescartesV2Created event

        expect(eventArgs["_input"], "input address").to.equal(
            await descartesV2Impl.input()
        );

        expect(eventArgs["_output"], "output address").to.equal(
            await descartesV2Impl.output()
        );

        expect(
            eventArgs["_validatorManager"],
            "Validator Manager address"
        ).to.equal(await descartesV2Impl.validatorManager());

        expect(
            eventArgs["_disputeManager"],
            "Dispute Manager address"
        ).to.equal(await descartesV2Impl.disputeManager());

        expect(eventArgs["_inputDuration"], "Input Duration").to.equal(
            inputDuration
        );

        expect(eventArgs["_challengePeriod"], "Challenge Period").to.equal(
            challengePeriod
        );
    });

    it("event Claim for NoConflict and Conflict", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                0,
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        await expect(
            descartesV2Impl
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("not hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                0,
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("not hello")
            );
    });

    it("event Claim for Consensus", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                0,
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        // skip input duration
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        // claim epoch 1
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                1,
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
    });

    it("event PhaseChange", async () => {
        //advance input duration from input accumulation start
        await network.provider.send("evm_increaseTime", [
            (await descartesV2Impl.getInputAccumulationStart()).toNumber() +
                inputDuration +
                1,
        ]);
        await network.provider.send("evm_mine");

        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "PhaseChange")
            .withArgs(Phase.AwaitingConsensus);

        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        //event PhaseChange: InputAccumulation
        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "PhaseChange")
            .withArgs(Phase.InputAccumulation);

        // @dev this version doesnt include Awaiting Dispute phase
        //event PhaseChange: AwaitingDispute
        //await expect(
        //    descartesV2Impl
        //        .connect(signers[1])
        //        .claim(ethers.utils.formatBytes32String("halo"))
        //)
        //    .to.emit(descartesV2Impl, "PhaseChange")
        //    .withArgs(Phase.AwaitingDispute);
    });

    it("event FinalizeEpoch", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await descartesV2Impl
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("hello"));

        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "FinalizeEpoch")
            .withArgs(0, ethers.utils.formatBytes32String("hello"));
    });

    /// modifiers off
    if (!permissionModifiersOn) {
        //event ResolveDispute needs to be tested without modifier: onlyDisputeContract
        it("event ResolveDispute", async () => {
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await expect(
                descartesV2Impl.resolveDispute(
                    await signers[0].getAddress(),
                    await signers[1].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                )
            )
                .to.emit(descartesV2Impl, "ResolveDispute")
                .withArgs(
                    await signers[0].getAddress(),
                    await signers[1].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                );
        });
    }

    it("getCurrentEpoch() without conflict", async () => {
        // initial epoch number
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(0);

        let epochNum = 0;

        // epoch number increases when input accumulation finishes
        // the length of finalized epochs array increases upon consensus without conflict
        for (let i = 0; i < 9; i++) {
            // input accumulation
            expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

            // input accumulation over
            // ***epoch increases by 1***
            // but output.getNumberOfFinalizedEpochs() stays the same temporarily
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");
            epochNum++;

            await expect(
                descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
            )
                .to.emit(descartesV2Impl, "Claim")
                .withArgs(
                    epochNum - 1, // claim for the previous epoch
                    await signers[0].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                );

            await descartesV2Impl
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("hello"));

            expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

            await expect(
                descartesV2Impl
                    .connect(signers[2])
                    .claim(ethers.utils.formatBytes32String("hello"))
            )
                .to.emit(descartesV2Impl, "Claim")
                .withArgs(
                    epochNum - 1, // claim for the previous epoch
                    await signers[2].getAddress(),
                    ethers.utils.formatBytes32String("hello")
                );
            // enter input accumulation again
            // ***the length of finalized epochs array increases by 1***
            // now it is the same as the epoch number
            expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);
        }
    });

    it("getCurrentEpoch() with conflict", async () => {
        // initial epoch number
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(0);

        let epochNum = 0;

        // input accumulation
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

        // input accumulation over
        // ***epoch increases by 1***
        // but output.getNumberOfFinalizedEpochs() stays the same temporarily
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");
        epochNum++;

        // first claim
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

        // 2nd claim => conflict
        await expect(
            descartesV2Impl
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("halo"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("halo")
            );

        // 3rd claim => Consensus
        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        // enter input accumulation again
        // ***the length of finalized epochs array increases by 1***
        // now it is the same as the epoch number
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

        // in this epoch, signers[1] is already deleted
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");
        epochNum++;
        // first claim
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);

        // 2nd claim => revert because claimer lost the dispute before
        await expect(
            descartesV2Impl
                .connect(signers[1])
                .claim(ethers.utils.formatBytes32String("hello"))
        ).to.be.revertedWith("sender not allowed");

        // 3rd claim => Consensus
        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                epochNum - 1, // claim for the previous epoch
                await signers[2].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

        // enter input accumulation again
        expect(await descartesV2Impl.getCurrentEpoch()).to.equal(epochNum);
    });

    // test delegate
    if (enableDelegate) {
        /* example DescartesV2 delegate output looks like 
        {
            constants: {
                input_duration: '0x15180',
                challenge_period: '0x93a80',
                contract_creation_timestamp: '0x616e3ac3',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81',
                output_contract_address: '0x6d544390eb535d61e196c87d6b9c80dcd8628acd',
                validator_contract_address: '0xb1ede3f5ac8654124cb5124adf0fd3885cbdd1f7',
                dispute_contract_address: '0xa6d6d7c556ce6ada136ba32dbe530993f128ca44',
                descartesv2_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9'
            },
            initial_epoch: '0x0',
            finalized_epochs: {
                finalized_epochs: [],
                initial_epoch: '0x0',
                descartesv2_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
            },
            current_epoch: {
                epoch_number: '0x0',
                inputs: {
                epoch_number: '0x0',
                inputs: [],
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
                },
                descartesv2_contract_address: '0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9',
                input_contract_address: '0xd8058efe0198ae9dd7d563e1b4938dcbc86a1f81'
            },
            current_phase: { InputAccumulation: {} },
            output_state: {
                output_address: '0x6d544390eb535d61e196c87d6b9c80dcd8628acd',
                outputs: {}
            }
        }
        */

        it("test delegate", async () => {
            let state = JSON.parse(await getState(initialState));

            // *** initial test ***

            // test constants
            expect(
                parseInt(state.constants.input_duration, 16),
                "check input duration"
            ).to.equal(inputDuration);
            expect(
                parseInt(state.constants.challenge_period, 16),
                "check challenge period"
            ).to.equal(challengePeriod);
            expect(
                parseInt(state.constants.contract_creation_timestamp, 16),
                "check contract creation timestamp"
            ).to.equal(contract_creation_time);
            expect(
                state.constants.input_contract_address,
                "check input contract address"
            ).to.equal((await descartesV2Impl.getInputAddress()).toLowerCase());
            expect(
                state.constants.output_contract_address,
                "check output contract address"
            ).to.equal(
                (await descartesV2Impl.getOutputAddress()).toLowerCase()
            );
            expect(
                state.constants.validator_contract_address,
                "check validator manager contract address"
            ).to.equal(
                (
                    await descartesV2Impl.getValidatorManagerAddress()
                ).toLowerCase()
            );
            expect(
                state.constants.dispute_contract_address,
                "check dispute manager contract address"
            ).to.equal(
                (await descartesV2Impl.getDisputeManagerAddress()).toLowerCase()
            );
            expect(
                state.constants.descartesv2_contract_address,
                "check descartesV2 contract address"
            ).to.equal(descartesV2Impl.address.toLowerCase());

            // test initial_epoch
            expect(state.initial_epoch, "check initial epoch").to.equal(
                initialEpoch
            );

            // test initial finalized_epochs
            expect(
                state.finalized_epochs.finalized_epochs.length,
                "check initial finalized_epochs.finalized_epochs"
            ).to.equal(0);
            expect(
                state.finalized_epochs.initial_epoch,
                "check finalized_epochs.initial_epoch"
            ).to.equal(initialEpoch);
            expect(
                state.finalized_epochs.descartesv2_contract_address,
                "check finalized_epochs.descartesv2_contract_address"
            ).to.equal(descartesV2Impl.address.toLowerCase());
            expect(
                state.finalized_epochs.input_contract_address,
                "check finalized_epochs.input_contract_address"
            ).to.equal((await descartesV2Impl.getInputAddress()).toLowerCase());

            // test initial current_epoch
            expect(
                state.current_epoch.epoch_number,
                "check initial current_epoch.epoch_number"
            ).to.equal(initialEpoch);
            expect(
                state.current_epoch.inputs.epoch_number,
                "check initial current_epoch.inputs.epoch_number"
            ).to.equal(initialEpoch);
            expect(
                state.current_epoch.inputs.inputs.length,
                "initially there's no inputs"
            ).to.equal(0);
            expect(
                state.current_epoch.inputs.input_contract_address,
                "check current_epoch.inputs.input_contract_address"
            ).to.equal((await descartesV2Impl.getInputAddress()).toLowerCase());
            expect(
                state.current_epoch.descartesv2_contract_address,
                "check current_epoch.descartesv2_contract_address"
            ).to.equal(descartesV2Impl.address.toLowerCase());
            expect(
                state.current_epoch.input_contract_address,
                "check current_epoch.input_contract_address"
            ).to.equal((await descartesV2Impl.getInputAddress()).toLowerCase());
            expect(
                JSON.stringify(state.current_phase.InputAccumulation) == "{}",
                "initial phase"
            ).to.equal(true);
            expect(
                state.output_state.output_address,
                "check output_state.output_address"
            ).to.equal(
                (await descartesV2Impl.getOutputAddress()).toLowerCase()
            );
            expect(
                JSON.stringify(state.output_state.outputs) == "{}",
                "initially there's no outputs"
            ).to.equal(true);

            // *** EPOCH 0: claim when the input duration has not past ***
            await expect(
                descartesV2Impl.claim(
                    ethers.utils.formatBytes32String("hello")
                ),
                "phase incorrect because inputDuration not over"
            ).to.be.revertedWith("Phase != AwaitingConsensus");
            await network.provider.send("evm_increaseTime", [
                inputDuration / 2,
            ]);
            await network.provider.send("evm_mine");
            await expect(
                descartesV2Impl.claim(
                    ethers.utils.formatBytes32String("hello")
                ),
                "phase incorrect because inputDuration not over"
            ).to.be.revertedWith("Phase != AwaitingConsensus");

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                "InputAccumulation" in state.current_phase,
                "current phase should still be InputAccumulation"
            ).to.equal(true);

            // *** EPOCH 0: input duration has past, now make a claim ***
            await network.provider.send("evm_increaseTime", [
                inputDuration / 2 + 1,
            ]);
            await network.provider.send("evm_mine");
            await descartesV2Impl.claim(
                ethers.utils.formatBytes32String("hello")
            );

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                parseInt(state.current_epoch.epoch_number, 16),
                "inputDuration for epoch 0 has past, now accumulating inputs for epoch 1"
            ).to.equal(1);
            expect(
                "AwaitingConsensusNoConflict" in state.current_phase,
                "someone has claimed, now the phase should be AwaitingConsensusNoConflict"
            ).to.equal(true);
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
                "check the value of the claim"
            ).to.equal(true);
            expect(
                state.current_phase.AwaitingConsensusNoConflict.claimed_epoch
                    .claims.claims[
                    ethers.utils.formatBytes32String("hello")
                ][0],
                "check the sender address of the claim"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusNoConflict
                        .claimed_epoch.claims.first_claim_timestamp,
                    16
                ),
                "check the timestamp of the first claim"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);
            // inputs are tested in the input delegate tests

            // *** EPOCH 0: claim to reach consensus ***
            await descartesV2Impl
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

            await descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("hello"));

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                state.finalized_epochs.finalized_epochs[0].epoch_number,
                "finalized epoch number"
            ).to.equal("0x0");
            expect(
                state.finalized_epochs.finalized_epochs[0].hash,
                "finalized hash"
            ).to.equal(ethers.utils.formatBytes32String("hello"));
            // inputs are tested in the input delegate tests
            expect(
                state.finalized_epochs.finalized_epochs[0].finalized_block_hash,
                "check finalized_block_hash"
            ).to.equal((await ethers.provider.getBlock("latest")).hash);
            expect(
                parseInt(
                    state.finalized_epochs.finalized_epochs[0]
                        .finalized_block_number,
                    16
                ),
                "check finalized_block_number"
            ).to.equal((await ethers.provider.getBlock("latest")).number);

            expect(
                state.current_epoch.epoch_number,
                "since epoch 0 just finalized, the current epoch should still be 1"
            ).to.equal("0x1");
            expect(
                "InputAccumulation" in state.current_phase,
                "same reason as above, current phase should be InputAccumulation"
            ).to.equal(true);

            // *** EPOCH 1: sealed epoch ***
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                state.current_epoch.epoch_number,
                "input duration for epoch 1 has past, now accumulating inputs for epoch 2"
            ).to.equal("0x2");
            expect(
                "EpochSealedAwaitingFirstClaim" in state.current_phase,
                "now the Epoch 1 is sealed"
            ).to.equal(true);
            expect(
                state.current_phase.EpochSealedAwaitingFirstClaim.sealed_epoch
                    .epoch_number,
                "check the sealed epoch number"
            ).to.equal("0x1");

            // *** EPOCH 1: conflicting claims ***
            await descartesV2Impl.claim(
                ethers.utils.formatBytes32String("hello1")
            );
            let first_claim_timestamp = (
                await ethers.provider.getBlock("latest")
            ).timestamp;
            await descartesV2Impl
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
                "check address of the first claim"
            ).to.equal((await signers[0].getAddress()).toLowerCase());
            expect(
                state.current_phase.AwaitingConsensusAfterConflict.claimed_epoch
                    .claims.claims[
                    ethers.utils.formatBytes32String("not hello1")
                ][0],
                "check address of the challenging claim"
            ).to.equal((await signers[1].getAddress()).toLowerCase());
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusAfterConflict
                        .claimed_epoch.claims.first_claim_timestamp,
                    16
                ),
                "check timestamp of the first claim"
            ).to.equal(first_claim_timestamp);
            expect(
                parseInt(
                    state.current_phase.AwaitingConsensusAfterConflict
                        .challenge_period_base_ts,
                    16
                ),
                "check timestamp of the challenging claim"
            ).to.equal((await ethers.provider.getBlock("latest")).timestamp);

            // *** EPOCH 1: consensus waiting period times out ***
            await network.provider.send("evm_increaseTime", [
                challengePeriod + 1,
            ]);
            await network.provider.send("evm_mine");

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                "ConsensusTimeout" in state.current_phase,
                "current phase should be ConsensusTimeout"
            ).to.equal(true);
            expect(
                state.current_phase.ConsensusTimeout.claimed_epoch.epoch_number,
                "epoch number when ConsensusTimeout"
            ).to.equal("0x1");

            // *** EPOCH 1 -> 2: finalize after consensus times out ***
            await descartesV2Impl.finalizeEpoch();

            state = JSON.parse(await getState(initialState)); // update state
            // now can test the finalized epoch 1
            expect(
                state.finalized_epochs.finalized_epochs[1].epoch_number,
                "finalized epoch number 1"
            ).to.equal("0x1");
            expect(
                state.finalized_epochs.finalized_epochs[1].hash,
                "finalized hash for epoch 1"
            ).to.equal(ethers.utils.formatBytes32String("hello1"));
            expect(
                state.finalized_epochs.finalized_epochs[1].finalized_block_hash,
                "check finalized_block_hash for epoch 1"
            ).to.equal((await ethers.provider.getBlock("latest")).hash);
            expect(
                parseInt(
                    state.finalized_epochs.finalized_epochs[1]
                        .finalized_block_number,
                    16
                ),
                "check finalized_block_number"
            ).to.equal((await ethers.provider.getBlock("latest")).number);

            expect(
                "InputAccumulation" in state.current_phase,
                "current phase should be InputAccumulation for epoch 2"
            ).to.equal(true);

            // *** EPOCH 2 -> 3: conflicting claims but reach consensus once conflict is resolved ***
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await descartesV2Impl.claim(
                ethers.utils.formatBytes32String("hello2")
            );
            await descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("not hello2"));

            state = JSON.parse(await getState(initialState)); // update state
            expect(
                state.current_epoch.epoch_number,
                "input duration for epoch 2 has past, now accumulating inputs for epoch 3"
            ).to.equal("0x3");
            expect(
                "InputAccumulation" in state.current_phase,
                "InputAccumulation of epoch 3"
            ).to.equal(true);

            expect(
                state.finalized_epochs.finalized_epochs[2].epoch_number,
                "finalized epoch number 2"
            ).to.equal("0x2");
            expect(
                state.finalized_epochs.finalized_epochs[2].hash,
                "finalized hash for epoch 2"
            ).to.equal(ethers.utils.formatBytes32String("hello2"));
            expect(
                state.finalized_epochs.finalized_epochs[2].finalized_block_hash,
                "check finalized_block_hash for epoch 2"
            ).to.equal((await ethers.provider.getBlock("latest")).hash);
            expect(
                parseInt(
                    state.finalized_epochs.finalized_epochs[2]
                        .finalized_block_number,
                    16
                ),
                "check finalized_block_number"
            ).to.equal((await ethers.provider.getBlock("latest")).number);
        });
    }
});
