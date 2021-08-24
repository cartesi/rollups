import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { DescartesV2Impl } from "../src/types/DescartesV2Impl";
import { DescartesV2Impl__factory } from "../src/types/factories/DescartesV2Impl__factory";
import {
    formatBytes32String,
    parseBytes32String,
} from "@ethersproject/strings";
import { ProjectPathsUserConfig } from "hardhat/types";

use(solidity);

describe("Descartes V2 Implementation", () => {
    /// for testing DescartesV2 when modifiers are on, set this to true
    /// for testing DescartesV2 when modifiers are off, set this to false
    let permissionModifiersOn = true;

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

    beforeEach(async () => {
        signers = await ethers.getSigners();

        await deployments.fixture();

        /* comment these if we are not using deploy script
        const dAddress = (await deployments.get("DescartesV2Impl")).address;
        descartesV2Impl = DescartesV2Impl__factory.connect(
            dAddress,
            signers[0]
        );
        */

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

        // may need to resolve types conflict
        // DescartesV2Impl
        // const descartesV2Impl_factory = await ethers.getContractFactory("DescartesV2Impl",
        // {
        //     signer: signers[0],
        //     libraries:{
        //         Bitmask: bitMaskAddress,
        //         Merkle: merkleAddress,
        //     }
        // } )
        // descartesV2Impl = await descartesV2Impl_factory.deploy(
        //     inputDuration,
        //     challengePeriod,
        //     INPUT_LOG2_SIZE,
        //     OUTPUT_METADATA_LOG2_SIZE,
        //     [
        //         await signers[0].getAddress(),
        //         await signers[1].getAddress(),
        //         await signers[2].getAddress(),
        //     ]
        // );
        // await descartesV2Impl.deployed();

        const { address } = await deployments.deploy("DescartesV2Impl", {
            from: await signers[0].getAddress(),
            libraries: {
                Bitmask: bitMaskAddress,
                Merkle: merkleAddress,
            },
            args: [
                inputDuration,
                challengePeriod,
                INPUT_LOG2_SIZE,
                OUTPUT_METADATA_LOG2_SIZE,
                [
                    await signers[0].getAddress(),
                    await signers[1].getAddress(),
                    await signers[2].getAddress(),
                ],
            ],
        });
        descartesV2Impl = DescartesV2Impl__factory.connect(address, signers[0]);
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
        ).to.be.revertedWith("Challenge period is not over");
    });

    it("claim(): should revert if the current claim is null", async () => {
        let currentClaim = ethers.utils.formatBytes32String("\0");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await expect(
            descartesV2Impl.claim(currentClaim),
            "cannot claim 0x00"
        ).to.be.revertedWith("claim cannot be 0x00");
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
            ).to.be.revertedWith("msg.sender != input contract");
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
            ).to.be.revertedWith("msg.sender != dispute manager contract");
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
        ).to.be.revertedWith("_sender was not allowed to claim");

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
});
