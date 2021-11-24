import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer, Contract } from "ethers";
import { getState } from "./getState";

use(solidity);

describe("Diamond", () => {
    /// for testing Rollups when modifiers are on, set this to true
    /// for testing Rollups when modifiers are off, set this to false
    let permissionModifiersOn = true;

    let enableDelegate = process.env["DELEGATE_TEST"];

    let diamondAddress : string;
    let validatorManagerFacet : Contract;

    const MINUTE = 60; // seconds in a minute
    const HOUR = 60 * MINUTE; // seconds in an hour
    const DAY = 24 * HOUR; // seconds in a day

    const inputDuration = 1 * DAY;
    const challengePeriod = 7 * DAY;
    const INPUT_LOG2_SIZE = 25;
    const VOUCHER_METADATA_LOG2_SIZE = 21;

    let signers: Signer[];

    let hash_zero = ethers.constants.HashZero;

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
        diamondAddress = (await deployments.get("CartesiRollups")).address;
        validatorManagerFacet = await ethers.getContractAt('ValidatorManagerFacet', diamondAddress);
    });

    /// ***test public variable currentPhase*** ///
    it("initial current claim check", async () => {
        expect(
            await validatorManagerFacet.getCurrentClaim(),
            "initial current claim check"
        ).to.equal(hash_zero);
    });

});
