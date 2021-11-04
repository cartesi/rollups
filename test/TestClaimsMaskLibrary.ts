import { deployments, ethers, network } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import { TestClaimsMaskLibrary } from "../dist/src/types/TestClaimsMaskLibrary";
import { TestClaimsMaskLibrary__factory } from "../dist/src/types/factories/TestClaimsMaskLibrary__factory";

use(solidity);

describe("Test ClaimsMaskLibrary", () => {
    let signers: Signer[];
    let claimsMaskLibrary: TestClaimsMaskLibrary;

    beforeEach(async () => {
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

    it("create a numClaimsRedeemed", async () => {
        // case 1
        let numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            100, 0, 0, 0, 0, 0, 0, 0,
        ]);
        expect(numClaimsRedeemed, "new numClaimsRedeemed").to.equal(100);

        // case 2
        let hexValue =
            "0x0000000700000006000000050000000400000003000000020000000100000000";
        numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            0, 1, 2, 3, 4, 5, 6, 7,
        ]);
        expect(numClaimsRedeemed, "another new numClaimsRedeemed").to.equal(
            hexValue
        );

        // case 3,4,5 - revert
        await expect(
            claimsMaskLibrary.newNumClaimsRedeemed([
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                parseInt("0x100000000", 16),
            ]),
            "0x100000000 is out of range (rear)"
        ).to.be.revertedWith("value out of range");

        await expect(
            claimsMaskLibrary.newNumClaimsRedeemed([
                parseInt("0x100000000", 16),
                1,
                2,
                3,
                4,
                5,
                6,
                7,
            ]),
            "0x100000000 is out of range (front)"
        ).to.be.revertedWith("value out of range");

        await expect(
            claimsMaskLibrary.newNumClaimsRedeemed([
                0,
                1,
                2,
                parseInt("0x100000000", 16),
                4,
                5,
                6,
                7,
            ]),
            "0x100000000 is out of range (middle)"
        ).to.be.revertedWith("value out of range");
    });

    it("test getNumClaimsRedeemed", async () => {
        // create a claimsMask with the following value
        // "0x0000000700000006000000050000000400000003000000020000000100000000"
        let numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            0, 1, 2, 3, 4, 5, 6, 7,
        ]);
        for (let i = 0; i < 8; i++) {
            expect(
                await claimsMaskLibrary.getNumClaimsRedeemed(
                    numClaimsRedeemed,
                    i
                ),
                "get #ClaimsRedeemed on index i"
            ).to.equal(i);
        }
    });

    it("test setNumClaimsRedeemed", async () => {
        // create a claimsMask with the following value
        // "0x0000000700000006000000050000000400000003000000020000000100000000"
        let hexValue =
            "0x0000000700000006000000050000000400000003000000020000000100000000";
        let numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            0, 1, 2, 3, 4, 5, 6, 7,
        ]);

        // set #ClaimsRedeemed to the same as it is
        for (let i = 0; i < 8; i++) {
            numClaimsRedeemed = await claimsMaskLibrary.setNumClaimsRedeemed(
                numClaimsRedeemed,
                i,
                i
            );
        }
        expect(numClaimsRedeemed, "still the same").to.equal(hexValue);

        // set #ClaimsRedeemed all to 0
        for (let i = 0; i < 8; i++) {
            numClaimsRedeemed = await claimsMaskLibrary.setNumClaimsRedeemed(
                numClaimsRedeemed,
                i,
                0
            );
        }
        expect(numClaimsRedeemed, "set all to 0").to.equal(0);

        // set all to the max value
        for (let i = 0; i < 8; i++) {
            numClaimsRedeemed = await claimsMaskLibrary.setNumClaimsRedeemed(
                numClaimsRedeemed,
                i,
                "0xFFFFFFFF"
            );
        }
        expect(numClaimsRedeemed, "set all to max").to.equal(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
        );

        // overflow test
        for (let i = 0; i < 8; i++) {
            await expect(
                claimsMaskLibrary.setNumClaimsRedeemed(
                    numClaimsRedeemed,
                    i,
                    "0x100000000"
                ),
                "set to overflow because 100000000 > FFFFFFFF"
            ).to.be.revertedWith("ClaimMask Overflow");
        }
    });

    it("test increaseNumClaimed", async () => {
        // create a claimsMask with the following value
        // "0x0000000700000006000000050000000400000003000000020000000100000000"
        let hexValue =
            "0x0000000700000006000000050000000400000003000000020000000100000000";
        let numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            0, 1, 2, 3, 4, 5, 6, 7,
        ]);

        // increase #ClaimsRedeemed to the same as it is
        for (let i = 0; i < 8; i++) {
            numClaimsRedeemed = await claimsMaskLibrary.increaseNumClaimed(
                numClaimsRedeemed,
                i,
                0
            );
        }
        expect(numClaimsRedeemed, "increase by 0, still the same").to.equal(
            hexValue
        );

        // increase each entry by 16, or 0x10
        for (let i = 0; i < 8; i++) {
            numClaimsRedeemed = await claimsMaskLibrary.increaseNumClaimed(
                numClaimsRedeemed,
                i,
                16
            );
        }
        expect(numClaimsRedeemed, "all increased by 0x10").to.equal(
            "0x0000001700000016000000150000001400000013000000120000001100000010"
        );

        // overflow test
        for (let i = 0; i < 8; i++) {
            await expect(
                claimsMaskLibrary.increaseNumClaimed(
                    numClaimsRedeemed,
                    i,
                    "0xFFFFFFFF"
                ),
                "increase to overflow"
            ).to.be.revertedWith("ClaimMask Overflow");
        }
    });

    it("index out of range test", async () => {
        // create a claimsMask with the following value
        // "0x0000000700000006000000050000000400000003000000020000000100000000"
        let numClaimsRedeemed = await claimsMaskLibrary.newNumClaimsRedeemed([
            0, 1, 2, 3, 4, 5, 6, 7,
        ]);

        // getNumClaimsRedeemed
        await expect(
            claimsMaskLibrary.getNumClaimsRedeemed(numClaimsRedeemed, 8),
            "get #ClaimsRedeemed on index 8 - revert"
        ).to.be.revertedWith("index out of range");

        // increaseNumClaimed
        await expect(
            claimsMaskLibrary.increaseNumClaimed(numClaimsRedeemed, 8, 0),
            "increase #ClaimsRedeemed on index 8 - revert"
        ).to.be.revertedWith("index out of range");

        // setNumClaimsRedeemed
        await expect(
            claimsMaskLibrary.setNumClaimsRedeemed(numClaimsRedeemed, 8, 0),
            "set #ClaimsRedeemed on index 8 - revert"
        ).to.be.revertedWith("index out of range");
    });
});
