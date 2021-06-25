import { deployments, ethers } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import { OutputImpl } from "../src/types/OutputImpl";
import { Merkle } from "../src/types/Merkle";
import { CartesiMath } from "../src/types/CartesiMath";
import { Bytes, BytesLike } from "@ethersproject/bytes";
import { keccak256 } from "ethers/lib/utils";

use(solidity);

// In the test epoch, we have 2 inputs. For Input1, we have only Output0.
// The payload that we use is to execute functions of DescartesV2.
// However, the address of DescartesV2 may change on different machines.
// So you may need to modify the value of `outputMetadataArrayDriveHash` and `epochOutputDriveHash`, for both v and v2

// Steps for modification are as follows:
// 1. uncomment lines `console.log(encodedOutput);`. Same for `console.log(encodedOutput2);`
// 2. keccak256 the value of encodedOutput
// 3. take the keccak value and replace into the variable `KeccakForOutput0` in "shell.sh"
// 4. run the script and modify values of `outputMetadataArrayDriveHash` and `epochOutputDriveHash`

describe("Output Implementation Testing", () => {
    /// for tests when modifiers are on, set this to true
    /// for tests when modifiers are off, set this to false
    let permissionModifiersOn = true;

    let signers: Signer[];
    let mockDescartesV2: MockContract;
    let firstMockDescartesV2: MockContract;
    let outputImpl: OutputImpl;
    let merkle: Merkle;
    let cartesiMath: CartesiMath;

    const log2OutputMetadataArrayDriveSize = 21;

    let _destination: string;
    let _payload: string;
    let encodedOutput: string;

    beforeEach(async () => {
        // get signers
        signers = await ethers.getSigners();

        // mock DescartesV2
        const DescartesV2 = await deployments.getArtifact("DescartesV2");
        mockDescartesV2 = await deployMockContract(signers[0], DescartesV2.abi);

        // deploy libraries and get addresses. Depedencies are as follows:
        // OutputImpl <= Bitmask, Merkle
        // Merkle <= CartesiMath

        // Bitmask
        const bitMaskLibrary = await deployments.deploy("Bitmask", {
            from: await signers[0].getAddress(),
        });
        const bitMaskAddress = bitMaskLibrary.address;

        // CartesiMath
        const cartesiMathFactory = await ethers.getContractFactory(
            "CartesiMath",
            {
                signer: signers[0],
            }
        );
        cartesiMath = await cartesiMathFactory.deploy();
        const cartesiMathAddress = cartesiMath.address;

        // Merkle
        const merkleFactory = await ethers.getContractFactory("Merkle", {
            signer: signers[0],
            libraries: {
                CartesiMath: cartesiMathAddress,
            },
        });
        merkle = await merkleFactory.deploy();
        const merkleAddress = merkle.address;

        // link to libraries and deploy OutputImpl
        const outputImplFactory = await ethers.getContractFactory(
            "OutputImpl",
            {
                signer: signers[0],
                libraries: {
                    Bitmask: bitMaskAddress,
                    Merkle: merkleAddress,
                },
            }
        );

        // keep outputImpl associated with the first deployed mockDescartesV2
        if (!firstMockDescartesV2) {
            outputImpl = await outputImplFactory.deploy(
                mockDescartesV2.address,
                log2OutputMetadataArrayDriveSize
            );
        } else {
            outputImpl = await outputImplFactory.deploy(
                firstMockDescartesV2.address,
                log2OutputMetadataArrayDriveSize
            );
        }
    });

    interface OutputValidityProof {
        epochIndex: number;
        inputIndex: number;
        outputIndex: number;
        outputMetadataArrayDriveHash: BytesLike;
        epochOutputDriveHash: BytesLike;
        epochMessageDriveHash: BytesLike;
        epochMachineFinalState: BytesLike;
        outputMetadataProof: BytesLike[];
        epochOutputDriveProof: BytesLike[];
    }
    // proofs are from bottom to top
    let proof1 = [
        "0xae39ce8537aca75e2eff3e38c98011dfe934e700a0967732fc07b430dd656a23",
        "0x3fc9a15f5b4869c872f81087bb6104b7d63e6f9ab47f2c43f3535eae7172aa7f",
        "0x17d2dd614cddaa4d879276b11e0672c9560033d3e8453a1d045339d34ba601b9",
        "0xc37b8b13ca95166fb7af16988a70fcc90f38bf9126fd833da710a47fb37a55e6",
        "0x8e7a427fa943d9966b389f4f257173676090c6e95f43e2cb6d65f8758111e309",
        "0x30b0b9deb73e155c59740bacf14a6ff04b64bb8e201a506409c3fe381ca4ea90",
        "0xcd5deac729d0fdaccc441d09d7325f41586ba13c801b7eccae0f95d8f3933efe",
        "0xd8b96e5b7f6f459e9cb6a2f41bf276c7b85c10cd4662c04cbbb365434726c0a0",
        "0xc9695393027fb106a8153109ac516288a88b28a93817899460d6310b71cf1e61",
        "0x63e8806fa0d4b197a259e8c3ac28864268159d0ac85f8581ca28fa7d2c0c03eb",
        "0x91e3eee5ca7a3da2b3053c9770db73599fb149f620e3facef95e947c0ee860b7",
        "0x2122e31e4bbd2b7c783d79cc30f60c6238651da7f0726f767d22747264fdb046",
        "0xf7549f26cc70ed5e18baeb6c81bb0625cb95bb4019aeecd40774ee87ae29ec51",
        "0x7a71f6ee264c5d761379b3d7d617ca83677374b49d10aec50505ac087408ca89",
        "0x2b573c267a712a52e1d06421fe276a03efb1889f337201110fdc32a81f8e1524",
        "0x99af665835aabfdc6740c7e2c3791a31c3cdc9f5ab962f681b12fc092816a62f",
    ];
    let proof2 = [
        "0xf887dff6c734c5faf153d9788f64b984b92da62147d64fcd219a7862c9e3144f",
        "0x633dc4d7da7256660a892f8f1604a44b5432649cc8ec5cb3ced4c4e6ac94dd1d",
        "0x890740a8eb06ce9be422cb8da5cdafc2b58c0a5e24036c578de2a433c828ff7d",
        "0x3b8ec09e026fdc305365dfc94e189a81b38c7597b3d941c279f042e8206e0bd8",
        "0xecd50eee38e386bd62be9bedb990706951b65fe053bd9d8a521af753d139e2da",
        "0xdefff6d330bb5403f63b14f33b578274160de3a50df4efecf0e0db73bcdd3da5",
        "0x617bdd11f7c0a11f49db22f629387a12da7596f9d1704d7465177c63d88ec7d7",
        "0x292c23a9aa1d8bea7e2435e555a4a60e379a5a35f3f452bae60121073fb6eead",
        "0xe1cea92ed99acdcb045a6726b2f87107e8a61620a232cf4d7d5b5766b3952e10",
        "0x7ad66c0a68c72cb89e4fb4303841966e4062a76ab97451e3b9fb526a5ceb7f82",
        "0xe026cc5a4aed3c22a58cbd3d2ac754c9352c5436f638042dca99034e83636516",
        "0x3d04cffd8b46a874edf5cfae63077de85f849a660426697b06a829c70dd1409c",
        "0xad676aa337a485e4728a0b240d92b3ef7b3c372d06d189322bfd5f61f1e7203e",
        "0xa2fca4a49658f9fab7aa63289c91b7c7b6c832a6d0e69334ff5b0a3483d09dab",
        "0x4ebfd9cd7bca2505f7bef59cc1c12ecc708fff26ae4af19abe852afe9e20c862",
        "0x2def10d13dd169f550f578bda343d9717a138562e0093b380a1120789d53cf10",
        "0x776a31db34a1a0a7caaf862cffdfff1789297ffadc380bd3d39281d340abd3ad",
        "0xe2e7610b87a5fdf3a72ebe271287d923ab990eefac64b6e59d79f8b7e08c46e3",
        "0x504364a5c6858bf98fff714ab5be9de19ed31a976860efbd0e772a2efe23e2e0",
        "0x4f05f4acb83f5b65168d9fef89d56d4d77b8944015e6b1eed81b0238e2d0dba3",
        "0x44a6d974c75b07423e1d6d33f481916fdd45830aea11b6347e700cd8b9f0767c",
        "0xedf260291f734ddac396a956127dde4c34c0cfb8d8052f88ac139658ccf2d507",
        "0x6075c657a105351e7f0fce53bc320113324a522e8fd52dc878c762551e01a46e",
        "0x6ca6a3f763a9395f7da16014725ca7ee17e4815c0ff8119bf33f273dee11833b",
        "0x1c25ef10ffeb3c7d08aa707d17286e0b0d3cbcb50f1bd3b6523b63ba3b52dd0f",
        "0xfffc43bd08273ccf135fd3cacbeef055418e09eb728d727c4d5d5c556cdea7e3",
        "0xc5ab8111456b1f28f3c7a0a604b4553ce905cb019c463ee159137af83c350b22",
        "0x0ff273fcbf4ae0f2bd88d6cf319ff4004f8d7dca70d4ced4e74d2c74139739e6",
        "0x7fa06ba11241ddd5efdc65d4e39c9f6991b74fd4b81b62230808216c876f827c",
        "0x7e275adf313a996c7e2950cac67caba02a5ff925ebf9906b58949f3e77aec5b9",
        "0x8f6162fa308d2b3a15dc33cffac85f13ab349173121645aedf00f471663108be",
        "0x78ccaaab73373552f207a63599de54d7d8d0c1805f86ce7da15818d09f4cff62",
    ];
    let v: OutputValidityProof = {
        epochIndex: 0,
        inputIndex: 1,
        outputIndex: 0,
        outputMetadataArrayDriveHash:
            "0xd75942974f7b054c28d5da4fb27898cd4b6b4f08a4ed85763e6c91bc91b01c53",
        epochOutputDriveHash:
            "0x7b508f3a93aac8a034fedc755ed13640cf698898d53160a9b914b8f3c4971a18",
        epochMessageDriveHash:
            "0x143ab4b3ff53d0459e30790af7010a68c2d2a1b34b6bc440c4b53e8a16286d45",
        epochMachineFinalState:
            "0x143ab4b3ff53d0459e30790af7010a68c2d2a1b34b6bc440c4b53e8a16286d46",
        outputMetadataProof: proof1,
        epochOutputDriveProof: proof2,
    };
    // let encodedOutput='0xa6eb2a81043a7349b2d066b3433ceadd8dd290343e6c41a4e36e82261e0b25cb';
    let epochHash = keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                v.epochOutputDriveHash,
                v.epochMessageDriveHash,
                v.epochMachineFinalState,
            ]
        )
    );

    const iface = new ethers.utils.Interface([
        "function claim(bytes32) public",
        "function getCurrentEpoch() view returns (uint256)",
    ]);

    it("First test", async () => {
        // this test should be the first test in order to assign value to firstMockDescartesV2
        // because the address of mockDescartesV2 may keep changing
        firstMockDescartesV2 = mockDescartesV2;

        // in the following test cases, we mainly use DescartesV2.getCurrentEpoch() as an example of payload
        _destination = firstMockDescartesV2.address;
        _payload = iface.encodeFunctionData("getCurrentEpoch");
        encodedOutput = ethers.utils.defaultAbiCoder.encode(
            ["uint", "bytes"],
            [_destination, _payload]
        );
        // console.log(encodedOutput);
    });

    if (!permissionModifiersOn) {
        it("payload to execute DescartesV2.getCurrentEpoch()", async () => {
            // example of encodedOutput
            // 0x
            // 0000000000000000000000005fbdb2315678afecb367f032d93f642f64180aa3
            // 0000000000000000000000000000000000000000000000000000000000000040
            // 0000000000000000000000000000000000000000000000000000000000000004
            // b97dd9e200000000000000000000000000000000000000000000000000000000

            // onNewEpoch() should be called first to push some epochHashes before calling executeOutput()
            // we need permission modifier off to call onNewEpoch()
            await outputImpl.onNewEpoch(epochHash);
            await firstMockDescartesV2.mock.getCurrentEpoch.returns(1);
            expect(
                await outputImpl.callStatic.executeOutput(
                    _destination,
                    _payload,
                    v
                )
            ).to.equal(true);
        });

        it("executeOutput() should revert if output has already been executed", async () => {
            await outputImpl.onNewEpoch(epochHash);
            await firstMockDescartesV2.mock.getCurrentEpoch.returns(1);
            await outputImpl.executeOutput(_destination, _payload, v);
            await expect(
                outputImpl.executeOutput(_destination, _payload, v)
            ).to.be.revertedWith("output has already been executed");
        });

        it("payload to execute DescartesV2.claim() with failure and then success", async () => {
            let _payload2 = iface.encodeFunctionData("claim(bytes32)", [
                ethers.utils.formatBytes32String("hello"),
            ]);
            let encodedOutput2 = ethers.utils.defaultAbiCoder.encode(
                ["uint", "bytes"],
                [_destination, _payload2]
            );
            // console.log(encodedOutput2);
            let v2 = Object.assign({}, v); // copy object contents from v to v2, rather than just the address
            v2.outputMetadataArrayDriveHash =
                "0x280d8252a72ec8495d9a42f001252f05296227ed44c5ee27dc2c3b4a8bf9aee6";
            v2.epochOutputDriveHash =
                "0x939e8c5cbb201b94ebd65938869f52eb0a667f92f4bc7517651807db30028a92";
            let epochHash2 = keccak256(
                ethers.utils.defaultAbiCoder.encode(
                    ["uint", "uint", "uint"],
                    [
                        v2.epochOutputDriveHash,
                        v2.epochMessageDriveHash,
                        v2.epochMachineFinalState,
                    ]
                )
            );
            // onNewEpoch() should be called first to push some epochHashes before calling executeOutput()
            await outputImpl.onNewEpoch(epochHash2);
            // this will fail because claim() is not mocked yet
            expect(
                await outputImpl.callStatic.executeOutput(
                    _destination,
                    _payload2,
                    v2
                )
            ).to.equal(false);

            await firstMockDescartesV2.mock.claim.returns();
            expect(
                await outputImpl.callStatic.executeOutput(
                    _destination,
                    _payload2,
                    v2
                )
            ).to.equal(true);
        });
    }

    /// ***test function isValidProof()///
    it("testing function isValidProof()", async () => {
        expect(
            await outputImpl.isValidProof(encodedOutput, epochHash, v)
        ).to.equal(true);
    });

    it("isValidProof() should revert when _epochHash doesn't match", async () => {
        await expect(
            outputImpl.isValidProof(
                encodedOutput,
                ethers.utils.formatBytes32String("hello"),
                v
            )
        ).to.be.revertedWith(
            "epoch outputs hash is not represented in the epoch hash"
        );
    });

    it("isValidProof() should revert when epochOutputDriveHash doesn't match", async () => {
        let tempInputIndex = v.inputIndex;
        v.inputIndex = 10;
        await expect(
            outputImpl.isValidProof(encodedOutput, epochHash, v)
        ).to.be.revertedWith(
            "output's metadata drive hash is not contained in epoch output drive"
        );
        v.inputIndex = tempInputIndex;
    });

    it("isValidProof() should revert when outputMetadataArrayDriveHash doesn't match", async () => {
        let tempOutputIndex = v.outputIndex;
        v.outputIndex = 10;
        await expect(
            outputImpl.isValidProof(encodedOutput, epochHash, v)
        ).to.be.revertedWith(
            "specific output is not contained in output metadata drive hash"
        );
        v.outputIndex = tempOutputIndex;
    });

    /// ***test function getBitMaskPosition()*** ///
    it("testing function getBitMaskPosition()", async () => {
        const _output = 123;
        const _input = 456;
        const _epoch = 789;
        expect(
            await outputImpl.getBitMaskPosition(_output, _input, _epoch)
        ).to.equal(
            BigInt(_output) * BigInt(2 ** 128) +
                BigInt(_input) * BigInt(2 ** 64) +
                BigInt(_epoch)
        );
    });

    /// ***test function getIntraDrivePosition()*** ///
    it("testing function getIntraDrivePosition()", async () => {
        const _index = 10;
        const _log2Size = 4;
        expect(
            await outputImpl.getIntraDrivePosition(_index, _log2Size)
        ).to.equal(_index * 2 ** _log2Size);
    });

    /// ***test function getNumberOfFinalizedEpochs() and onNewEpoch()*** ///
    it("testing initial return value of getNumberOfFinalizedEpochs()", async () => {
        expect(await outputImpl.getNumberOfFinalizedEpochs()).to.equal(0);
    });

    if (permissionModifiersOn) {
        it("only DescartesV2 can call onNewEpoch()", async () => {
            await expect(
                outputImpl.onNewEpoch(ethers.utils.formatBytes32String("hello"))
            ).to.be.revertedWith("Only descartesV2 can call this function");
        });
    }

    if (!permissionModifiersOn) {
        it("fake onNewEpoch() to test if getNumberOfFinalizedEpochs() increases", async () => {
            await outputImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello")
            );
            expect(await outputImpl.getNumberOfFinalizedEpochs()).to.equal(1);

            await outputImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello2")
            );
            expect(await outputImpl.getNumberOfFinalizedEpochs()).to.equal(2);

            await outputImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello3")
            );
            expect(await outputImpl.getNumberOfFinalizedEpochs()).to.equal(3);
        });
    }

    /// ***test function getOutputMetadataLog2Size()*** ///
    it("testing function getOutputMetadataLog2Size()", async () => {
        expect(await outputImpl.getOutputMetadataLog2Size()).to.equal(21);
    });

    /// ***test function getEpochOutputLog2Size()*** ///
    it("testing function getEpochOutputLog2Size()", async () => {
        expect(await outputImpl.getEpochOutputLog2Size()).to.equal(37);
    });

    // /// ***test function executeOutput()*** ///
    // // onNewEpoch() should be called first to push some epochHashes before calling executeOutput()
    // // we need permission modifier off to call onNewEpoch()
    // if (!permissionModifiersOn) {
    //     it("testing initial return value of getNumberOfFinalizedEpochs()", async () => {
    //         // let executeOutput() execute the function getNumberOfFinalizedEpochs()
    //         const _destination = outputImpl.address;
    //         const abi = ["function getNumberOfFinalizedEpochs()"];
    //         const iface = new ethers.utils.Interface(abi);
    //         const _payload = iface.encodeFunctionData(
    //             "getNumberOfFinalizedEpochs"
    //         );

    //         // initialize parameters for executeOutput()
    //         const _epochIndex = 0;
    //         const _inputIndex = 1;
    //         const _outputIndex = 2;
    //         const hashOfOutput = ethers.utils.solidityKeccak256(
    //             ["string", "string"],
    //             [_destination, _payload]
    //         );
    //         const KECCAK_LOG2_SIZE = 5;
    //         let _outputProof = [];
    //         let _epochProof = [];
    //         for (let i = 0; i < 64 - KECCAK_LOG2_SIZE; i++) {
    //             // random text formed by numbers and letters
    //             // 10 numbers and 26 letters, thus base is 36
    //             let randomText = Math.random().toString(36).substring(2, 15);
    //             _outputProof.push(ethers.utils.formatBytes32String(randomText));
    //             randomText = Math.random().toString(36).substring(2, 15);
    //             _epochProof.push(ethers.utils.formatBytes32String(randomText));
    //         }
    //         let _outputDriveHash = await merkle.getRootWithDrive(
    //             _outputIndex * KECCAK_LOG2_SIZE,
    //             KECCAK_LOG2_SIZE,
    //             hashOfOutput,
    //             _outputProof
    //         );
    //         let epochHash0 = await merkle.getRootWithDrive(
    //             _inputIndex * KECCAK_LOG2_SIZE,
    //             KECCAK_LOG2_SIZE,
    //             _outputDriveHash,
    //             _epochProof
    //         );

    //         // push hash of epoch 0
    //         await outputImpl.onNewEpoch(epochHash0);

    //         //   expect(
    //         //     await outputImpl.executeOutput(
    //         //       _destination,
    //         //       _payload,
    //         //       _epochIndex,
    //         //       _inputIndex,
    //         //       _outputIndex,
    //         //       _outputDriveHash,
    //         //       _outputProof,
    //         //       _epochProof
    //         //     ),
    //         //     "executeOutput() to execute function getNumberOfFinalizedEpochs(). Should return 1"
    //         //   ).to.equal(1);
    //     });
    // }
});
