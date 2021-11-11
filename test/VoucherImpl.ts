import { deployments, ethers } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    deployMockContract,
    MockContract,
} from "@ethereum-waffle/mock-contract";
import { VoucherImpl } from "../src/types/VoucherImpl";
import { VoucherImpl__factory } from "../src/types/factories/VoucherImpl__factory";
import { Merkle } from "../src/types/Merkle";
import { CartesiMath } from "../src/types/CartesiMath";
import { Bytes, BytesLike } from "@ethersproject/bytes";
import { keccak256 } from "ethers/lib/utils";
import { getState } from "./getState";

use(solidity);

// In the test epoch, we have 2 inputs. For Input1, we have only Voucher0.
// The payload that we use is to execute functions of a simple contract.
// Normally, the address of that contract may change on different machines, resulting in different merkle proofs.
// That's why we deploy that contract deterministically.

// In case you need to modify proofs, modify the value of `voucherMetadataArrayDriveHash` and `epochVoucherDriveHash`

// Steps for modification are as follows:
// 1. uncomment lines `console.log(encodedVoucher);`.
// 2. keccak256 the value of encodedVoucher
// 3. take the keccak value and replace into the variable `KeccakForVoucher0` in "shell.sh"
// 4. run the script and modify values of `voucherMetadataArrayDriveHash` and `epochVoucherDriveHash`

describe("Voucher Implementation Testing", () => {
    let enableDelegate = process.env["DELEGATE_TEST"];

    /// for tests when modifiers are on, set this to true
    /// for tests when modifiers are off, set this to false
    let permissionModifiersOn = true;

    let signers: Signer[];
    let mockRollups: MockContract;
    let voucherImpl: VoucherImpl;

    const log2VoucherMetadataArrayDriveSize = 21;

    let simpleContractAddress: string;
    let _destination: string;
    let _payload: string;
    let encodedVoucher: string;

    beforeEach(async () => {
        await deployments.fixture();
        // get signers
        signers = await ethers.getSigners();

        // mock Rollups
        const Rollups = await deployments.getArtifact("Rollups");
        mockRollups = await deployMockContract(signers[0], Rollups.abi);

        // deploy libraries and get addresses. Depedencies are as follows:
        // VoucherImpl <= Bitmask, Merkle
        // Merkle <= CartesiMath

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

        // VoucherImpl
        const { address } = await deployments.deploy("VoucherImpl", {
            from: await signers[0].getAddress(),
            libraries: {
                Bitmask: bitMaskAddress,
                Merkle: merkleAddress,
            },
            args: [mockRollups.address, log2VoucherMetadataArrayDriveSize],
        });
        voucherImpl = VoucherImpl__factory.connect(address, signers[0]);

        // deploy a simple contract to execute
        const simpleContract = await deployments.deploy("SimpleContract", {
            from: await signers[0].getAddress(),
            deterministicDeployment: true, // deployed address is calculated based on contract bytecode and constructor arguments
        });
        simpleContractAddress = simpleContract.address;
    });

    interface VoucherValidityProof {
        epochIndex: number;
        inputIndex: number;
        voucherIndex: number;
        voucherMetadataArrayDriveHash: BytesLike;
        epochVoucherDriveHash: BytesLike;
        epochMessageDriveHash: BytesLike;
        epochMachineFinalState: BytesLike;
        voucherMetadataProof: BytesLike[];
        epochVoucherDriveProof: BytesLike[];
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
    let v: VoucherValidityProof = {
        epochIndex: 0,
        inputIndex: 1,
        voucherIndex: 0,
        voucherMetadataArrayDriveHash:
            "0x84842b18ffa0a3ca497e8eb68fc326792d9141482b3cefdeee05b6c7639ccdfb",
        epochVoucherDriveHash:
            "0x2157a942123603e60e93a226bf1cad66c4258675632697359d6845c186ff47d4",
        epochMessageDriveHash:
            "0x143ab4b3ff53d0459e30790af7010a68c2d2a1b34b6bc440c4b53e8a16286d45",
        epochMachineFinalState:
            "0x143ab4b3ff53d0459e30790af7010a68c2d2a1b34b6bc440c4b53e8a16286d46",
        voucherMetadataProof: proof1,
        epochVoucherDriveProof: proof2,
    };
    let epochHash = keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["uint", "uint", "uint"],
            [
                v.epochVoucherDriveHash,
                v.epochMessageDriveHash,
                v.epochMachineFinalState,
            ]
        )
    );

    const iface = new ethers.utils.Interface([
        "function simple_function() public pure returns (string memory)",
        "function simple_function(bytes32) public pure returns (string memory)",
        "function nonExistent() public",
    ]);

    it("Initialization", async () => {
        _destination = simpleContractAddress;
        _payload = iface.encodeFunctionData("simple_function()");
        encodedVoucher = ethers.utils.defaultAbiCoder.encode(
            ["uint", "bytes"],
            [_destination, _payload]
        );
        // console.log(encodedVoucher);
        // example of encodedVoucher
        // 0x
        // 0000000000000000000000005fbdb2315678afecb367f032d93f642f64180aa3
        // 0000000000000000000000000000000000000000000000000000000000000040
        // 0000000000000000000000000000000000000000000000000000000000000004
        // b97dd9e200000000000000000000000000000000000000000000000000000000

        expect(
            await voucherImpl.getNumberOfFinalizedEpochs(),
            "check initial epoch number"
        ).to.equal(0);
    });

    if (!permissionModifiersOn) {
        // disable modifiers to call onNewEpoch()
        it("executeVoucher(): execute SimpleContract.simple_function()", async () => {
            // onNewEpoch() should be called first to push some epochHashes before calling executeVoucher()
            // we need permission modifier off to call onNewEpoch()
            await voucherImpl.onNewEpoch(epochHash);
            expect(
                await voucherImpl.callStatic.executeVoucher(
                    _destination,
                    _payload,
                    v
                )
            ).to.equal(true);
        });

        it("executeVoucher(): execute SimpleContract.simple_function(bytes32)", async () => {
            let _payload_new = iface.encodeFunctionData(
                "simple_function(bytes32)",
                [ethers.utils.formatBytes32String("hello")]
            );
            let encodedVoucher_new = ethers.utils.defaultAbiCoder.encode(
                ["uint", "bytes"],
                [_destination, _payload_new]
            );
            // console.log(encodedVoucher_new);

            let v_new = Object.assign({}, v); // copy object contents from v to v_new, rather than just the address reference
            v_new.voucherMetadataArrayDriveHash =
                "0xc26ccca0f2995d3584e183ff7d8e2cd9f6ac01e263a3beb8f1a2345638d2bc9c";
            v_new.epochVoucherDriveHash =
                "0x4ddc5a9a0f46871a08135296b981b86a8bca580c7f7d7de7c473089f234abab1";
            let epochHash_new = keccak256(
                ethers.utils.defaultAbiCoder.encode(
                    ["uint", "uint", "uint"],
                    [
                        v_new.epochVoucherDriveHash,
                        v_new.epochMessageDriveHash,
                        v_new.epochMachineFinalState,
                    ]
                )
            );

            await voucherImpl.onNewEpoch(epochHash_new);
            expect(
                await voucherImpl.callStatic.executeVoucher(
                    _destination,
                    _payload_new,
                    v_new
                )
            ).to.equal(true);
        });

        it("executeVoucher(): should return false if the function to be executed failed (in this case the function does NOT exist)", async () => {
            let _payload_new = iface.encodeFunctionData("nonExistent()");
            let encodedVoucher_new = ethers.utils.defaultAbiCoder.encode(
                ["uint", "bytes"],
                [_destination, _payload_new]
            );
            // console.log(encodedVoucher_new);

            let v_new = Object.assign({}, v); // copy object contents from v to v_new, rather than just the address reference
            v_new.voucherMetadataArrayDriveHash =
                "0x9bfa174d480e37b808e0bf8ac3f2c5e4e25113c435dec2e353370fe956c3cb10";
            v_new.epochVoucherDriveHash =
                "0x5b0d4f6b91fdfe5eebe393a19ee03426def24e061f78b6cb57dd19ac9e5404e8";
            let epochHash_new = keccak256(
                ethers.utils.defaultAbiCoder.encode(
                    ["uint", "uint", "uint"],
                    [
                        v_new.epochVoucherDriveHash,
                        v_new.epochMessageDriveHash,
                        v_new.epochMachineFinalState,
                    ]
                )
            );

            await voucherImpl.onNewEpoch(epochHash_new);
            expect(
                await voucherImpl.callStatic.executeVoucher(
                    _destination,
                    _payload_new,
                    v_new
                )
            ).to.equal(false);
        });

        it("executeVoucher() should revert if voucher has already been executed", async () => {
            await voucherImpl.onNewEpoch(epochHash);
            await voucherImpl.executeVoucher(_destination, _payload, v);
            await expect(
                voucherImpl.executeVoucher(_destination, _payload, v)
            ).to.be.revertedWith("re-execution not allowed");
        });

        it("executeVoucher() should revert if proof is not valid", async () => {
            await voucherImpl.onNewEpoch(epochHash);
            let _payload_new = iface.encodeFunctionData("nonExistent()");
            await expect(
                voucherImpl.executeVoucher(_destination, _payload_new, v)
            ).to.be.reverted;
        });
    }

    /// ***test function isValidProof()///
    it("testing function isValidProof()", async () => {
        expect(
            await voucherImpl.isValidProof(encodedVoucher, epochHash, v)
        ).to.equal(true);
    });

    it("isValidProof() should revert when _epochHash doesn't match", async () => {
        await expect(
            voucherImpl.isValidProof(
                encodedVoucher,
                ethers.utils.formatBytes32String("hello"),
                v
            )
        ).to.be.revertedWith("epochHash incorrect");
    });

    it("isValidProof() should revert when epochVoucherDriveHash doesn't match", async () => {
        let tempInputIndex = v.inputIndex;
        v.inputIndex = 10;
        await expect(
            voucherImpl.isValidProof(encodedVoucher, epochHash, v)
        ).to.be.revertedWith("epochVoucherDriveHash incorrect");
        // restore v
        v.inputIndex = tempInputIndex;
    });

    it("isValidProof() should revert when voucherMetadataArrayDriveHash doesn't match", async () => {
        let tempVoucherIndex = v.voucherIndex;
        v.voucherIndex = 10;
        await expect(
            voucherImpl.isValidProof(encodedVoucher, epochHash, v)
        ).to.be.revertedWith("voucherMetadataArrayDriveHash incorrect");
        // restore v
        v.voucherIndex = tempVoucherIndex;
    });

    /// ***test function getBitMaskPosition()*** ///
    it("testing function getBitMaskPosition()", async () => {
        const _voucher = 123;
        const _input = 456;
        const _epoch = 789;
        expect(
            await voucherImpl.getBitMaskPosition(_voucher, _input, _epoch)
        ).to.equal(
            BigInt(_voucher) * BigInt(2 ** 128) +
                BigInt(_input) * BigInt(2 ** 64) +
                BigInt(_epoch)
        );
    });

    /// ***test function getIntraDrivePosition()*** ///
    it("testing function getIntraDrivePosition()", async () => {
        const _index = 10;
        const _log2Size = 4;
        expect(
            await voucherImpl.getIntraDrivePosition(_index, _log2Size)
        ).to.equal(_index * 2 ** _log2Size);
    });

    /// ***test function getNumberOfFinalizedEpochs() and onNewEpoch()*** ///
    if (permissionModifiersOn) {
        it("only Rollups can call onNewEpoch()", async () => {
            await expect(
                voucherImpl.onNewEpoch(ethers.utils.formatBytes32String("hello"))
            ).to.be.revertedWith("Only rollups");
        });
    }

    if (!permissionModifiersOn) {
        it("fake onNewEpoch() to test if getNumberOfFinalizedEpochs() increases", async () => {
            await voucherImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello")
            );
            expect(await voucherImpl.getNumberOfFinalizedEpochs()).to.equal(1);

            await voucherImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello2")
            );
            expect(await voucherImpl.getNumberOfFinalizedEpochs()).to.equal(2);

            await voucherImpl.onNewEpoch(
                ethers.utils.formatBytes32String("hello3")
            );
            expect(await voucherImpl.getNumberOfFinalizedEpochs()).to.equal(3);
        });
    }

    /// ***test function getVoucherMetadataLog2Size()*** ///
    it("testing function getVoucherMetadataLog2Size()", async () => {
        expect(await voucherImpl.getVoucherMetadataLog2Size()).to.equal(21);
    });

    /// ***test function getEpochVoucherLog2Size()*** ///
    it("testing function getEpochVoucherLog2Size()", async () => {
        expect(await voucherImpl.getEpochVoucherLog2Size()).to.equal(37);
    });

    /// ***test delegate*** ///
    if (enableDelegate) {
        it("testing voucher delegate", async () => {
            /// ***test case 1 - initial check
            let initialState = JSON.stringify({
                voucher_address: voucherImpl.address,
            });
            let state = JSON.parse(await getState(initialState));

            // initial check, executed vouchers should be empty
            expect(
                JSON.stringify(state.vouchers) == "{}",
                "initial check"
            ).to.equal(true);

            if (!permissionModifiersOn) {
                // disable modifiers to call onNewEpoch()
                /// ***test case 2 - only one voucher executed
                // voucherIndex: 0;
                //  inputIndex: 1;
                //  epochIndex: 0;
                await voucherImpl.onNewEpoch(epochHash);
                await voucherImpl.executeVoucher(_destination, _payload, v);

                state = JSON.parse(await getState(initialState));

                // vouchers look like { '0': { '1': { '0': true } } }
                expect(
                    Object.keys(state.vouchers).length,
                    "should have 1 executed voucher"
                ).to.equal(1);
                expect(
                    state.vouchers[0][1][0],
                    "the first voucher is executed successfully"
                ).to.equal(true);

                /// ***test case 3 - execute another voucher
                // execute another voucher for epoch 1
                let _payload_new = iface.encodeFunctionData(
                    "simple_function(bytes32)",
                    [ethers.utils.formatBytes32String("hello")]
                );

                let v_new = Object.assign({}, v); // copy object contents from v to v_new, rather than just the address reference
                v_new.epochIndex = 1; // we use the same voucherIndex and inputIndex
                v_new.voucherMetadataArrayDriveHash =
                    "0xc26ccca0f2995d3584e183ff7d8e2cd9f6ac01e263a3beb8f1a2345638d2bc9c";
                v_new.epochVoucherDriveHash =
                    "0x4ddc5a9a0f46871a08135296b981b86a8bca580c7f7d7de7c473089f234abab1";
                let epochHash_new = keccak256(
                    ethers.utils.defaultAbiCoder.encode(
                        ["uint", "uint", "uint"],
                        [
                            v_new.epochVoucherDriveHash,
                            v_new.epochMessageDriveHash,
                            v_new.epochMachineFinalState,
                        ]
                    )
                );

                await voucherImpl.onNewEpoch(epochHash_new);
                await voucherImpl.executeVoucher(
                    _destination,
                    _payload_new,
                    v_new
                );

                state = JSON.parse(await getState(initialState));

                // vouchers look like { '0': { '1': { '0': true, '1': true } } }
                expect(
                    Object.keys(state.vouchers[0][1]).length,
                    "should have 2 executed voucher"
                ).to.equal(2);
                expect(
                    state.vouchers[0][1][1],
                    "execute the second voucher"
                ).to.equal(true);

                /// ***test case 4 - execute a non-existent function
                _payload_new = iface.encodeFunctionData("nonExistent()");
                v_new = Object.assign({}, v); // copy object contents from v to v_new, rather than just the address reference
                v_new.epochIndex = 2;
                v_new.voucherMetadataArrayDriveHash =
                    "0x9bfa174d480e37b808e0bf8ac3f2c5e4e25113c435dec2e353370fe956c3cb10";
                v_new.epochVoucherDriveHash =
                    "0x5b0d4f6b91fdfe5eebe393a19ee03426def24e061f78b6cb57dd19ac9e5404e8";
                epochHash_new = keccak256(
                    ethers.utils.defaultAbiCoder.encode(
                        ["uint", "uint", "uint"],
                        [
                            v_new.epochVoucherDriveHash,
                            v_new.epochMessageDriveHash,
                            v_new.epochMachineFinalState,
                        ]
                    )
                );

                await voucherImpl.onNewEpoch(epochHash_new);
                await voucherImpl.executeVoucher(
                    _destination,
                    _payload_new,
                    v_new
                );

                state = JSON.parse(await getState(initialState));

                // since the execution was failed, everything should remain the same
                expect(
                    Object.keys(state.vouchers).length,
                    "only 1 voucherIndex"
                ).to.equal(1);
                expect(
                    Object.keys(state.vouchers[0]).length,
                    "1 inputIndex"
                ).to.equal(1);
                expect(
                    Object.keys(state.vouchers[0][1]).length,
                    "2 epochIndex"
                ).to.equal(2);

                /// ***test case 5 - re-execute an already executed voucher
                /// ***and test case 6 - proof not valid
                /// after these 2 failure cases, the executed vouchers should remain the same
                await expect(
                    voucherImpl.executeVoucher(_destination, _payload, v),
                    "already executed, should revert"
                ).to.be.revertedWith("re-execution not allowed");

                _payload_new = iface.encodeFunctionData("nonExistent()");
                await expect(
                    voucherImpl.executeVoucher(_destination, _payload_new, v),
                    "proof not valid, should revert"
                ).to.be.reverted;

                state = JSON.parse(await getState(initialState));
                expect(
                    Object.keys(state.vouchers).length,
                    "still only 1 voucherIndex"
                ).to.equal(1);
                expect(
                    Object.keys(state.vouchers[0]).length,
                    "still 1 inputIndex"
                ).to.equal(1);
                expect(
                    Object.keys(state.vouchers[0][1]).length,
                    "still 2 epochIndex"
                ).to.equal(2);
            }
        });
    }
});
