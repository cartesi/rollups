import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
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

    ///mock a contract as if it is already implemented. Check Waffle for details
    let mockInput: MockContract;
    let mockOutput: MockContract;
    let mockValidatorManager: MockContract;
    let mockDisputeManager: MockContract;

    let descartesV2Impl: DescartesV2Impl;

    const inputLog2Size = 25; // What is a good number for this?
    const inputDuration = 100;
    const challengePeriod = 100;

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

        const Input = await deployments.getArtifact("Input");
        const Output = await deployments.getArtifact("Output");
        const ValidatorManager = await deployments.getArtifact(
            "ValidatorManager"
        );
        const DisputeManager = await deployments.getArtifact("DisputeManager");

        mockInput = await deployMockContract(signers[0], Input.abi);
        mockOutput = await deployMockContract(signers[0], Output.abi);
        mockValidatorManager = await deployMockContract(
            signers[0],
            ValidatorManager.abi
        );
        mockDisputeManager = await deployMockContract(
            signers[0],
            DisputeManager.abi
        );

        const descartesV2ImplFactory = new DescartesV2Impl__factory(signers[0]);

        descartesV2Impl = await descartesV2ImplFactory.deploy(
            mockOutput.address,
            mockValidatorManager.address,
            mockDisputeManager.address,
            inputDuration,
            challengePeriod,
            inputLog2Size

        );

        await mockOutput.mock.getNumberOfFinalizedEpochs.returns(
            numberOfFinalizedEpochs
        ); //this may be needed when emit events

        // this is needed when a claim forces input box swap
        await mockInput.mock.onNewInputAccumulation.returns();
    });

    /// ***test function currentPhase()*** ///
    it("initial phase should be InputAccumulation", async () => {
        expect(
            await descartesV2Impl.currentPhase(),
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

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        expect(
            await descartesV2Impl.currentPhase(),
            "current phase should be updated to AwaitingConsensus"
        ).to.equal(Phase.AwaitingConsensus);
    });

    it("should claim() and enter into InputAccumulation phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("hello"));
        await descartesV2Impl
            .connect(signers[2])
            .claim(ethers.utils.formatBytes32String("hello"));

        await mockValidatorManager.mock.onClaim.returns(
            Result.Consensus,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [await signers[0].getAddress(), address_zero]
        );
        await mockValidatorManager.mock.onNewEpoch.returns(
            ethers.utils.formatBytes32String("hello")
        );
        await mockOutput.mock.onNewEpoch.returns();
        await mockInput.mock.onNewEpoch.returns();
        await descartesV2Impl
            .connect(signers[3])
            .claim(ethers.utils.formatBytes32String("hello"));

        expect(
            await descartesV2Impl.currentPhase(),
            "current phase should be updated to InputAccumulation"
        ).to.equal(Phase.InputAccumulation);
    });

    it("should claim() and enter into AwaitingDispute phase", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await mockValidatorManager.mock.onClaim.returns(
            Result.Conflict,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("halo"),
            ],
            [await signers[0].getAddress(), await signers[1].getAddress()]
        );
        await mockDisputeManager.mock.initiateDispute.returns();
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("halo"));

        expect(
            await descartesV2Impl.currentPhase(),
            "current phase should be updated to AwaitingDispute"
        ).to.equal(Phase.AwaitingDispute);
    });

    it("two different claim() will enter into AwaitingDispute phase, should revert if there are more claims", async () => {
        ///make two different claims///
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await mockValidatorManager.mock.onClaim.returns(
            Result.Conflict,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("halo"),
            ],
            [await signers[0].getAddress(), await signers[1].getAddress()]
        );
        await mockDisputeManager.mock.initiateDispute.returns();
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("halo"));
        ///END: make two different claims///

        await expect(
            descartesV2Impl
                .connect(signers[2])
                .claim(ethers.utils.formatBytes32String("lol")),
            "phase is AwaitingDispute. should revert"
        ).to.be.revertedWith("Phase != AwaitingConsensus");
    });

    /// ***test function finalizeEpoch()*** ///
    it("finalizeEpoch(): should revert if currentPhase is InputAccumulation", async () => {
        await expect(
            descartesV2Impl.finalizeEpoch(),
            "phase incorrect"
        ).to.be.revertedWith("Phase != Awaiting Consensus");
    });

    it("finalizeEpoch(): should revert if currentPhase is AwaitingDispute", async () => {
        ///make two different claims///
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await mockValidatorManager.mock.onClaim.returns(
            Result.Conflict,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("halo"),
            ],
            [await signers[0].getAddress(), await signers[1].getAddress()]
        );
        await mockDisputeManager.mock.initiateDispute.returns();
        await descartesV2Impl
            .connect(signers[1])
            .claim(ethers.utils.formatBytes32String("halo"));
        ///END: make two different claims///

        await expect(
            descartesV2Impl.finalizeEpoch(),
            "phase incorrect"
        ).to.be.revertedWith("Phase != Awaiting Consensus");
    });

    it("finalizeEpoch(): should revert if challengePeriod is not over", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"));

        await expect(
            descartesV2Impl.finalizeEpoch(),
            "Challenge period is not over"
        ).to.be.revertedWith("Challenge period is not over");
    });

    it("finalizeEpoch(): should revert if the current claim is null", async () => {
        let currentClaim = ethers.utils.formatBytes32String("\0");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(currentClaim);

        await network.provider.send("evm_increaseTime", [challengePeriod + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.getCurrentClaim.returns(currentClaim);
        await expect(
            descartesV2Impl.finalizeEpoch(),
            "current claim is null"
        ).to.be.revertedWith("No Claim to be finalized");
    });

    it("after finalizeEpoch(), current phase should be InputAccumulation", async () => {
        let currentClaim = ethers.utils.formatBytes32String("hello");
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await descartesV2Impl.claim(currentClaim);

        await network.provider.send("evm_increaseTime", [challengePeriod + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.getCurrentClaim.returns(currentClaim);
        await mockValidatorManager.mock.onNewEpoch.returns(
            ethers.utils.formatBytes32String("hello")
        );
        await mockOutput.mock.onNewEpoch.returns();
        await mockInput.mock.onNewEpoch.returns();

        await descartesV2Impl.finalizeEpoch();

        expect(
            await descartesV2Impl.currentPhase(),
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
                await descartesV2Impl.currentPhase(),
                "the updated current phase"
            ).to.equal(Phase.AwaitingConsensus);
        });

        it("notifyInput(): should return false if currentPhase is AwaitingDispute", async () => {
            ///make two different claims///
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await mockValidatorManager.mock.onClaim.returns(
                Result.NoConflict,
                [
                    ethers.utils.formatBytes32String("\0"),
                    ethers.utils.formatBytes32String("\0"),
                ],
                [address_zero, address_zero]
            );
            await descartesV2Impl.claim(
                ethers.utils.formatBytes32String("hello")
            );

            await mockValidatorManager.mock.onClaim.returns(
                Result.Conflict,
                [
                    ethers.utils.formatBytes32String("hello"),
                    ethers.utils.formatBytes32String("halo"),
                ],
                [await signers[0].getAddress(), await signers[1].getAddress()]
            );
            await mockDisputeManager.mock.initiateDispute.returns();
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

        /// ***test function resolveDispute() without modifier*** ///
        it("resolveDispute(): if consensus, updated current phase should be InputAccumulation", async () => {
            await mockValidatorManager.mock.onDisputeEnd.returns(
                Result.Consensus,
                [
                    ethers.utils.formatBytes32String("hello"),
                    ethers.utils.formatBytes32String("\0"),
                ],
                [await signers[0].getAddress(), address_zero]
            );
            await mockValidatorManager.mock.onNewEpoch.returns(
                ethers.utils.formatBytes32String("hello")
            );
            await mockOutput.mock.onNewEpoch.returns();
            await mockInput.mock.onNewEpoch.returns();

            await descartesV2Impl.resolveDispute(
                await signers[0].getAddress(),
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

            expect(
                await descartesV2Impl.currentPhase(),
                "updated current phase if consensus"
            ).to.equal(Phase.InputAccumulation);
        });

        it("resolveDispute(): if NoConflict, updated current phase should be AwaitingConsensus", async () => {
            await mockValidatorManager.mock.onDisputeEnd.returns(
                Result.NoConflict,
                [
                    ethers.utils.formatBytes32String("\0"),
                    ethers.utils.formatBytes32String("\0"),
                ],
                [address_zero, address_zero]
            );

            await descartesV2Impl.resolveDispute(
                await signers[0].getAddress(),
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

            expect(
                await descartesV2Impl.currentPhase(),
                "updated current phase if consensus"
            ).to.equal(Phase.AwaitingConsensus);
        });

        it("resolveDispute(): if Conflict, updated current phase should be AwaitingDispute", async () => {
            await mockValidatorManager.mock.onDisputeEnd.returns(
                Result.Conflict,
                [
                    ethers.utils.formatBytes32String("hello"),
                    ethers.utils.formatBytes32String("hello"),
                ],
                [await signers[0].getAddress(), await signers[0].getAddress()]
            );
            await mockDisputeManager.mock.initiateDispute.returns();

            await descartesV2Impl.resolveDispute(
                await signers[0].getAddress(),
                await signers[1].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );

            expect(
                await descartesV2Impl.currentPhase(),
                "updated current phase if Conflict"
            ).to.equal(Phase.AwaitingDispute);
            //then start new dispute all over again
        });
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
            mockOutput.address
        );

        expect(
            eventArgs["_validatorManager"],
            "Validator Manager address"
        ).to.equal(mockValidatorManager.address);

        expect(
            eventArgs["_disputeManager"],
            "Dispute Manager address"
        ).to.equal(mockDisputeManager.address);

        expect(eventArgs["_inputDuration"], "Input Duration").to.equal(
            inputDuration
        );

        expect(eventArgs["_challengePeriod"], "Challenge Period").to.equal(
            challengePeriod
        );
    });

    it("event Claim", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );

        //TODO: Add test case for claims that have a different Result
        //      like consensus
        // if the Claim resulted on a new epoch the argument should be
        // numberOfFinalizedEpochs - 1, because the claim is for the epoch
        // that was just finished (which is now being counted as finalized)
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "Claim")
            .withArgs(
                numberOfFinalizedEpochs,
                await signers[0].getAddress(),
                ethers.utils.formatBytes32String("hello")
            );
    });

    it("event PhaseChange", async () => {
        //advance input duration from input accumulation start
        await network.provider.send("evm_increaseTime", [
            (await descartesV2Impl.inputAccumulationStart()).toNumber() +
                inputDuration +
                1,
        ]);
        await network.provider.send("evm_mine");

        //event PhaseChange: AwaitingConsensus
        await mockValidatorManager.mock.onClaim.returns(
            Result.NoConflict,
            [
                ethers.utils.formatBytes32String("\0"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [address_zero, address_zero]
        );
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "PhaseChange")
            .withArgs(Phase.AwaitingConsensus);

        //event PhaseChange: InputAccumulation
        await mockValidatorManager.mock.onClaim.returns(
            Result.Consensus,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [await signers[0].getAddress(), address_zero]
        );
        await mockValidatorManager.mock.onNewEpoch.returns(
            ethers.utils.formatBytes32String("hello")
        );
        await mockOutput.mock.onNewEpoch.returns();
        await mockInput.mock.onNewEpoch.returns();
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "PhaseChange")
            .withArgs(Phase.InputAccumulation);

        //advance input duration from input accumulation start
        await network.provider.send("evm_increaseTime", [
            (await descartesV2Impl.inputAccumulationStart()).toNumber() +
                inputDuration +
                1,
        ]);
        await network.provider.send("evm_mine");

        //event PhaseChange: AwaitingDispute
        await mockValidatorManager.mock.onClaim.returns(
            Result.Conflict,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("halo"),
            ],
            [await signers[0].getAddress(), await signers[1].getAddress()]
        );
        await mockDisputeManager.mock.initiateDispute.returns();
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("halo"))
        )
            .to.emit(descartesV2Impl, "PhaseChange")
            .withArgs(Phase.AwaitingDispute);
    });

    it("event FinalizeEpoch", async () => {
        await network.provider.send("evm_increaseTime", [inputDuration + 1]);
        await network.provider.send("evm_mine");

        await mockValidatorManager.mock.onClaim.returns(
            Result.Consensus,
            [
                ethers.utils.formatBytes32String("hello"),
                ethers.utils.formatBytes32String("\0"),
            ],
            [await signers[0].getAddress(), address_zero]
        );
        await mockValidatorManager.mock.onNewEpoch.returns(
            ethers.utils.formatBytes32String("hello")
        );
        await mockOutput.mock.onNewEpoch.returns();
        await mockInput.mock.onNewEpoch.returns();

        // numberOfFinalizedEpochs - 1 because the epoch is finalized
        // before the event is emitted.
        await expect(
            descartesV2Impl.claim(ethers.utils.formatBytes32String("hello"))
        )
            .to.emit(descartesV2Impl, "FinalizeEpoch")
            .withArgs(
                numberOfFinalizedEpochs,
                ethers.utils.formatBytes32String("hello")
            );
    });

    /// modifiers off
    if (!permissionModifiersOn) {
        //event ResolveDispute needs to be tested without modifier: onlyDisputeContract
        it("event ResolveDispute", async () => {
            await network.provider.send("evm_increaseTime", [
                inputDuration + 1,
            ]);
            await network.provider.send("evm_mine");

            await mockValidatorManager.mock.onDisputeEnd.returns(
                Result.Consensus,
                [
                    ethers.utils.formatBytes32String("hello"),
                    ethers.utils.formatBytes32String("\0"),
                ],
                [await signers[0].getAddress(), address_zero]
            );
            await mockValidatorManager.mock.onNewEpoch.returns(
                ethers.utils.formatBytes32String("hello")
            );
            await mockOutput.mock.onNewEpoch.returns();
            await mockInput.mock.onNewEpoch.returns();

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
});
