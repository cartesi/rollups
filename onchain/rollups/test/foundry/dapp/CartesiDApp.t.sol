// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title Cartesi DApp Test
pragma solidity ^0.8.8;

import {TestBase} from "../util/TestBase.sol";

import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {Proof} from "contracts/dapp/ICartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {OutputValidityProof, LibOutputValidation} from "contracts/library/LibOutputValidation.sol";
import {OutputEncoding} from "contracts/common/OutputEncoding.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {IERC721Receiver} from "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";

import {LibServerManager} from "../util/LibServerManager.sol";
import {SimpleConsensus} from "../util/SimpleConsensus.sol";
import {SimpleERC20} from "../util/SimpleERC20.sol";
import {SimpleERC721} from "../util/SimpleERC721.sol";
import {SimpleERC721Receiver} from "../util/SimpleERC721Receiver.sol";

import "forge-std/console.sol";

contract EtherReceiver {
    receive() external payable {}
}

// Outputs
// 0: notice 0xfafafafa
// 1: voucher ERC-20 transfer
// 2: voucher ETH withdrawal
// 3: voucher ERC-721 transfer

contract CartesiDAppTest is TestBase {
    using LibServerManager for LibServerManager.RawFinishEpochResponse;
    using LibServerManager for LibServerManager.Proof;

    error ProofNotFound(
        LibServerManager.OutputEnum outputEnum,
        uint256 inputIndex
    );

    CartesiDApp dapp;
    IConsensus consensus;
    IERC20 erc20Token;
    IERC721 erc721Token;
    IERC721Receiver erc721Receiver;

    struct Voucher {
        address destination;
        uint256 value;
        bytes payload;
    }

    LibServerManager.OutputEnum[] outputEnums;
    mapping(uint256 => Voucher) vouchers;
    mapping(uint256 => bytes) notices;

    bytes encodedFinishEpochResponse;

    uint256 constant initialSupply = 1000000;
    uint256 constant transferAmount = 7;
    uint256 constant tokenId = uint256(keccak256("tokenId"));
    address constant dappOwner = address(bytes20(keccak256("dappOwner")));
    address constant tokenOwner = address(bytes20(keccak256("tokenOwner")));
    address constant recipient = address(bytes20(keccak256("recipient")));
    address constant noticeSender = address(bytes20(keccak256("noticeSender")));
    bytes32 constant salt = keccak256("salt");
    bytes32 constant templateHash = keccak256("templateHash");

    event VoucherExecuted(uint256 voucherPosition);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
    event NewConsensus(IConsensus newConsensus);

    function setUp() public {
        deployContracts();
        generateOutputs();
        writeInputs();
        readFinishEpochResponse();
    }

    function testConstructorWithOwnerAsZeroAddress(
        bytes32 _templateHash
    ) public {
        vm.expectRevert("Ownable: new owner is the zero address");
        new CartesiDApp(consensus, address(0), _templateHash);
    }

    function testConstructor(address _owner, bytes32 _templateHash) public {
        vm.assume(_owner != address(0));

        // An OwnershipTransferred event is always emitted
        // by the Ownership contract constructor
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));

        // A second OwnershipTransferred event is also emitted
        // by the CartesiDApp contract contructor
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(this), _owner);

        // perform call to constructor
        dapp = new CartesiDApp(consensus, _owner, _templateHash);

        // check set values
        assertEq(address(dapp.getConsensus()), address(consensus));
        assertEq(dapp.owner(), _owner);
        assertEq(dapp.getTemplateHash(), _templateHash);
    }

    // test notices

    function testNoticeValidation(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        bytes memory notice = getNotice(0);
        Proof memory proof = setupNoticeProof(0, _inputIndex, _numInputsAfter);

        bool ret = validateNotice(notice, proof);
        assertEq(ret, true);

        // reverts if notice is incorrect
        bytes memory falseNotice = abi.encodePacked(bytes4(0xdeaddead));
        vm.expectRevert(
            LibOutputValidation.IncorrectOutputHashesRootHash.selector
        );
        validateNotice(falseNotice, proof);
    }

    // test vouchers

    function testExecuteVoucherAndEvent(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        // not able to execute voucher because dapp has 0 balance
        assertEq(erc20Token.balanceOf(address(dapp)), 0);
        assertEq(erc20Token.balanceOf(recipient), 0);
        bool success = executeVoucher(voucher, proof);
        assertEq(success, false);
        assertEq(erc20Token.balanceOf(address(dapp)), 0);
        assertEq(erc20Token.balanceOf(recipient), 0);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);
        assertEq(erc20Token.balanceOf(address(dapp)), dappInitBalance);
        assertEq(erc20Token.balanceOf(recipient), 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndexWithinInput,
                _inputIndex
            )
        );

        // perform call
        success = executeVoucher(voucher, proof);

        // check result
        assertEq(success, true);
        assertEq(
            erc20Token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(erc20Token.balanceOf(recipient), transferAmount);
    }

    function testRevertsReexecution(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);

        // 1st execution attempt should succeed
        bool success = executeVoucher(voucher, proof);
        assertEq(success, true);

        // 2nd execution attempt should fail
        vm.expectRevert(CartesiDApp.VoucherReexecutionNotAllowed.selector);
        executeVoucher(voucher, proof);

        // end result should be the same as executing successfully only once
        assertEq(
            erc20Token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(erc20Token.balanceOf(recipient), transferAmount);
    }

    function testWasVoucherExecuted(
        uint128 _inputIndex,
        uint128 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        // before executing voucher
        bool executed = dapp.wasVoucherExecuted(
            _inputIndex,
            proof.validity.outputIndexWithinInput
        );
        assertEq(executed, false);

        // execute voucher - failed
        bool success = executeVoucher(voucher, proof);
        assertEq(success, false);

        // `wasVoucherExecuted` should still return false
        executed = dapp.wasVoucherExecuted(
            _inputIndex,
            proof.validity.outputIndexWithinInput
        );
        assertEq(executed, false);

        // execute voucher - succeeded
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);
        success = executeVoucher(voucher, proof);
        assertEq(success, true);

        // after executing voucher, `wasVoucherExecuted` should return true
        executed = dapp.wasVoucherExecuted(
            _inputIndex,
            proof.validity.outputIndexWithinInput
        );
        assertEq(executed, true);
    }

    function testRevertsEpochHash(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        proof.validity.outputsEpochRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert(LibOutputValidation.IncorrectEpochHash.selector);
        executeVoucher(voucher, proof);
    }

    function testRevertsOutputsEpochRootHash(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        proof.validity.outputHashesRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert(
            LibOutputValidation.IncorrectOutputsEpochRootHash.selector
        );
        executeVoucher(voucher, proof);
    }

    function testRevertsOutputHashesRootHash(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, _numInputsAfter);

        proof.validity.outputIndexWithinInput = 0xdeadbeef;

        vm.expectRevert(
            LibOutputValidation.IncorrectOutputHashesRootHash.selector
        );
        executeVoucher(voucher, proof);
    }

    function testRevertsInputIndexOOB(uint256 _inputIndex) public {
        Voucher memory voucher = getVoucher(1);
        Proof memory proof = setupVoucherProof(1, _inputIndex, 0);

        // If the input index within epoch were 0, then there would be no way for the
        // input index in input box to be out of bounds because every claim is non-empty,
        // as it must contain at least one input
        assert(proof.validity.inputIndexWithinEpoch > 0);

        // This assumption aims to avoid an integer overflow in the CartesiDApp
        vm.assume(
            _inputIndex <=
                type(uint256).max - proof.validity.inputIndexWithinEpoch
        );

        // Calculate epoch hash from proof
        bytes32 epochHash = calculateEpochHash(proof.validity);

        // Mock consensus again to return a claim that spans only 1 input,
        // but we are registering a proof whose epoch input index is 1...
        // so the proof would succeed but the input would be out of bounds
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getClaim.selector),
            abi.encode(epochHash, _inputIndex, _inputIndex)
        );

        vm.expectRevert(
            LibOutputValidation.InputIndexOutOfClaimBounds.selector
        );
        executeVoucher(voucher, proof);
    }

    // test ether transfer

    function testEtherTransfer(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(2);
        Proof memory proof = setupVoucherProof(2, _inputIndex, _numInputsAfter);

        // not able to execute voucher because dapp has 0 balance
        assertEq(address(dapp).balance, 0);
        assertEq(address(recipient).balance, 0);
        bool success = executeVoucher(voucher, proof);
        assertEq(success, false);
        assertEq(address(dapp).balance, 0);
        assertEq(address(recipient).balance, 0);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.deal(address(dapp), dappInitBalance);
        assertEq(address(dapp).balance, dappInitBalance);
        assertEq(address(recipient).balance, 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndexWithinInput,
                _inputIndex
            )
        );

        // perform call
        success = executeVoucher(voucher, proof);

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(address(recipient).balance, transferAmount);

        // cannot execute the same voucher again
        vm.expectRevert(CartesiDApp.VoucherReexecutionNotAllowed.selector);
        executeVoucher(voucher, proof);
    }

    function testWithdrawEtherContract(
        uint256 _value,
        address _notDApp
    ) public {
        vm.assume(_value <= address(this).balance);
        vm.assume(_notDApp != address(dapp));
        address receiver = address(new EtherReceiver());

        // fund dapp
        vm.deal(address(dapp), _value);

        // withdrawEther cannot be called by anyone
        vm.expectRevert(CartesiDApp.OnlyDApp.selector);
        vm.prank(_notDApp);
        dapp.withdrawEther(receiver, _value);

        // withdrawEther can only be called by dapp itself
        uint256 preBalance = receiver.balance;
        vm.prank(address(dapp));
        dapp.withdrawEther(receiver, _value);
        assertEq(receiver.balance, preBalance + _value);
        assertEq(address(dapp).balance, 0);
    }

    function testWithdrawEtherEOA(
        uint256 _value,
        address _notDApp,
        uint256 _receiverSeed
    ) public {
        vm.assume(_notDApp != address(dapp));
        vm.assume(_value <= address(this).balance);

        // by deriving receiver from keccak-256, we avoid
        // collisions with precompiled contract addresses
        // assume receiver is not a contract
        address receiver = address(
            bytes20(keccak256(abi.encode(_receiverSeed)))
        );
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(receiver)
        }
        vm.assume(codeSize == 0);

        // fund dapp
        vm.deal(address(dapp), _value);

        // withdrawEther cannot be called by anyone
        vm.expectRevert(CartesiDApp.OnlyDApp.selector);
        vm.prank(_notDApp);
        dapp.withdrawEther(receiver, _value);

        // withdrawEther can only be called by dapp itself
        uint256 preBalance = receiver.balance;
        vm.prank(address(dapp));
        dapp.withdrawEther(receiver, _value);
        assertEq(receiver.balance, preBalance + _value);
        assertEq(address(dapp).balance, 0);
    }

    function testRevertsWithdrawEther(uint256 _value, uint256 _funds) public {
        vm.assume(_value > _funds);
        address receiver = address(new EtherReceiver());

        // Fund DApp
        vm.deal(address(dapp), _funds);

        // DApp is not funded or does not have enough funds
        vm.prank(address(dapp));
        vm.expectRevert(CartesiDApp.EtherTransferFailed.selector);
        dapp.withdrawEther(receiver, _value);
    }

    // test NFT transfer

    function testWithdrawNFT(
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) public {
        Voucher memory voucher = getVoucher(3);
        Proof memory proof = setupVoucherProof(3, _inputIndex, _numInputsAfter);

        // not able to execute voucher because dapp doesn't have the nft
        assertEq(erc721Token.ownerOf(tokenId), tokenOwner);
        bool success = executeVoucher(voucher, proof);
        assertEq(success, false);
        assertEq(erc721Token.ownerOf(tokenId), tokenOwner);

        // fund dapp
        vm.prank(tokenOwner);
        erc721Token.safeTransferFrom(tokenOwner, address(dapp), tokenId);
        assertEq(erc721Token.ownerOf(tokenId), address(dapp));

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndexWithinInput,
                _inputIndex
            )
        );

        // perform call
        success = executeVoucher(voucher, proof);

        // check result
        assertEq(success, true);
        assertEq(erc721Token.ownerOf(tokenId), address(erc721Receiver));

        // cannot execute the same voucher again
        vm.expectRevert(CartesiDApp.VoucherReexecutionNotAllowed.selector);
        executeVoucher(voucher, proof);
    }

    // test migration

    function testMigrateToConsensus(
        address _owner,
        bytes32 _templateHash,
        address _newOwner,
        address _nonZeroAddress
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));
        vm.assume(_owner != _newOwner);
        vm.assume(address(_newOwner) != address(0));
        vm.assume(_nonZeroAddress != address(0));

        dapp = new CartesiDApp(consensus, _owner, _templateHash);

        IConsensus newConsensus = new SimpleConsensus();

        // migrate fail if not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        dapp.migrateToConsensus(newConsensus);

        // now impersonate owner
        vm.prank(_owner);
        vm.expectEmit(false, false, false, true, address(dapp));
        emit NewConsensus(newConsensus);
        dapp.migrateToConsensus(newConsensus);
        assertEq(address(dapp.getConsensus()), address(newConsensus));

        // if owner changes, then original owner no longer can migrate consensus
        vm.prank(_owner);
        dapp.transferOwnership(_newOwner);
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(_owner);
        dapp.migrateToConsensus(consensus);

        // if new owner renounce ownership (give ownership to address 0)
        // no one will be able to migrate consensus
        vm.prank(_newOwner);
        dapp.renounceOwnership();
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(_nonZeroAddress);
        dapp.migrateToConsensus(consensus);
    }

    // Store proof in storage
    // Mock consensus so that calls to `getClaim` return
    // values that can be used to validate the proof.
    function registerProof(
        uint256 _inputIndex,
        uint256 _numInputsAfter,
        Proof memory _proof
    ) internal {
        // check if `_inputIndex` and `_numInputsAfter` are valid
        vm.assume(_proof.validity.inputIndexWithinEpoch <= _inputIndex);
        vm.assume(_numInputsAfter <= type(uint256).max - _inputIndex);

        // calculate epoch hash from proof
        bytes32 epochHash = calculateEpochHash(_proof.validity);

        // calculate input index range based on proof and fuzzy variables
        uint256 firstInputIndex = _inputIndex -
            _proof.validity.inputIndexWithinEpoch;
        uint256 lastInputIndex = _inputIndex + _numInputsAfter;

        // mock the consensus contract to return the right epoch hash
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getClaim.selector),
            abi.encode(epochHash, firstInputIndex, lastInputIndex)
        );
    }

    function deployContracts() internal {
        consensus = deployConsensusDeterministically();
        dapp = deployDAppDeterministically();
        erc20Token = deployERC20Deterministically();
        erc721Token = deployERC721Deterministically();
        erc721Receiver = deployERC721ReceiverDeterministically();
    }

    function deployDAppDeterministically() internal returns (CartesiDApp) {
        vm.prank(dappOwner);
        return new CartesiDApp{salt: salt}(consensus, dappOwner, templateHash);
    }

    function deployConsensusDeterministically() internal returns (IConsensus) {
        vm.prank(dappOwner);
        return new SimpleConsensus{salt: salt}();
    }

    function deployERC20Deterministically() internal returns (IERC20) {
        vm.prank(tokenOwner);
        return new SimpleERC20{salt: salt}(tokenOwner, initialSupply);
    }

    function deployERC721Deterministically() internal returns (IERC721) {
        vm.prank(tokenOwner);
        return new SimpleERC721{salt: salt}(tokenOwner, tokenId);
    }

    function deployERC721ReceiverDeterministically()
        internal
        returns (IERC721Receiver)
    {
        vm.prank(tokenOwner);
        return new SimpleERC721Receiver{salt: salt}();
    }

    function addVoucher(
        address destination,
        uint256 value,
        bytes memory payload
    ) internal {
        uint256 index = outputEnums.length;
        outputEnums.push(LibServerManager.OutputEnum.VOUCHER);
        vouchers[index] = Voucher(destination, value, payload);
    }

    function addVoucher(
        address destination,
        bytes memory payload
    ) internal {
        addVoucher(destination, 0, payload);
    }

    function getVoucher(
        uint256 inputIndex
    ) internal view returns (Voucher memory) {
        assert(outputEnums[inputIndex] == LibServerManager.OutputEnum.VOUCHER);
        return vouchers[inputIndex];
    }

    function addNotice(
        bytes memory notice
    ) internal {
        uint256 index = outputEnums.length;
        outputEnums.push(LibServerManager.OutputEnum.NOTICE);
        notices[index] = notice;
    }

    function getNotice(
        uint256 inputIndex
    ) internal view returns (bytes memory) {
        assert(outputEnums[inputIndex] == LibServerManager.OutputEnum.NOTICE);
        return notices[inputIndex];
    }

    function generateOutputs() internal {
        addNotice(abi.encode(bytes4(0xfafafafa)));
        addVoucher(address(erc20Token), abi.encodeWithSelector(
            IERC20.transfer.selector,
            recipient,
            transferAmount
        ));
        addVoucher(address(dapp), abi.encodeWithSelector(
            CartesiDApp.withdrawEther.selector,
            recipient,
            transferAmount
        ));
        addVoucher(address(erc721Token), abi.encodeWithSignature(
            "safeTransferFrom(address,address,uint256)",
            dapp,
            erc721Receiver,
            tokenId
        ));
    }

    function encodeVoucher(
        Voucher calldata voucher
    ) external pure returns (bytes memory) {
        return OutputEncoding.encodeVoucher(voucher.destination, voucher.value, voucher.payload);
    }

    function encodeNotice(
        bytes calldata notice
    ) external pure returns (bytes memory) {
        return OutputEncoding.encodeNotice(notice);
    }

    function writeInputs() internal {
        for (uint256 i; i < outputEnums.length; ++i) {
            LibServerManager.OutputEnum outputEnum = outputEnums[i];
            if (outputEnum == LibServerManager.OutputEnum.VOUCHER) {
                Voucher memory voucher = getVoucher(i);
                writeInput(i, noticeSender, this.encodeVoucher(voucher));
            } else {
                bytes memory notice = getNotice(i);
                writeInput(i, noticeSender, this.encodeNotice(notice));
            }
        }
    }

    function writeInput(
        uint256 inputIndex,
        address sender,
        bytes memory payload
    ) internal {
        string memory inputIndexStr = vm.toString(inputIndex);
        string memory objectKey = string.concat("input", inputIndexStr);
        vm.serializeAddress(objectKey, "sender", sender);
        string memory json = vm.serializeBytes(objectKey, "payload", payload);
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test",
            "/foundry",
            "/dapp",
            "/helper",
            "/input",
            "/",
            inputIndexStr,
            ".json"
        );
        vm.writeJson(json, path);
    }

    function readFinishEpochResponse() internal {
        // Construct path to FinishEpoch response JSON
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test",
            "/foundry",
            "/dapp",
            "/helper",
            "/output",
            "/finish_epoch_response.json"
        );

        // Read contents of JSON file
        string memory json = vm.readFile(path);

        // Parse JSON into ABI-encoded data
        encodedFinishEpochResponse = vm.parseJson(json);
    }

    function validateNotice(
        bytes memory notice,
        Proof memory proof
    ) internal view returns (bool) {
        return dapp.validateNotice(notice, proof);
    }

    function executeVoucher(
        Voucher memory voucher,
        Proof memory proof
    ) internal returns (bool) {
        return dapp.executeVoucher(voucher.destination, voucher.payload, proof);
    }

    function calculateEpochHash(
        OutputValidityProof memory _validity
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    _validity.outputsEpochRootHash,
                    _validity.machineStateHash
                )
            );
    }

    function setupNoticeProof(
        uint256 _inputIndexWithinEpoch,
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) internal returns (Proof memory) {
        Proof memory proof = getNoticeProof(_inputIndexWithinEpoch);
        registerProof(_inputIndex, _numInputsAfter, proof);
        return proof;
    }

    function setupVoucherProof(
        uint256 _inputIndexWithinEpoch,
        uint256 _inputIndex,
        uint256 _numInputsAfter
    ) internal returns (Proof memory) {
        Proof memory proof = getNoticeProof(_inputIndexWithinEpoch);
        registerProof(_inputIndex, _numInputsAfter, proof);
        return proof;
    }

    function getNoticeProof(
        uint256 inputIndex
    ) internal view returns (Proof memory) {
        return getProof(LibServerManager.OutputEnum.NOTICE, inputIndex, 0);
    }

    function getVoucherProof(
        uint256 inputIndex
    ) internal view returns (Proof memory) {
        return getProof(LibServerManager.OutputEnum.VOUCHER, inputIndex, 0);
    }

    function getProof(
        LibServerManager.OutputEnum outputEnum,
        uint256 inputIndex,
        uint256 outputIndex
    ) internal view returns (Proof memory) {
        // Decode ABI-encoded data into raw struct
        LibServerManager.RawFinishEpochResponse memory raw = abi.decode(
            encodedFinishEpochResponse,
            (LibServerManager.RawFinishEpochResponse)
        );

        // Format raw finish epoch response
        LibServerManager.FinishEpochResponse memory response = raw.fmt(vm);

        // Find the proof that proves the provided output
        LibServerManager.Proof[] memory proofs = response.proofs;
        for (uint256 i; i < proofs.length; ++i) {
            LibServerManager.Proof memory proof = proofs[i];
            if (proof.proves(outputEnum, inputIndex, outputIndex)) {
                return convert(proof);
            }
        }

        // If a proof was not found, raise an error
        revert ProofNotFound(outputEnum, inputIndex);
    }

    function convert(
        LibServerManager.OutputValidityProof memory v
    ) internal pure returns (OutputValidityProof memory) {
        return
            OutputValidityProof({
                inputIndexWithinEpoch: uint64(v.inputIndexWithinEpoch),
                outputIndexWithinInput: uint64(v.outputIndexWithinInput),
                outputHashesRootHash: v.outputHashesRootHash,
                outputsEpochRootHash: v.noticesEpochRootHash,
                machineStateHash: v.machineStateHash,
                outputHashInOutputHashesSiblings: v
                    .outputHashInOutputHashesSiblings,
                outputHashesInEpochSiblings: v.outputHashesInEpochSiblings
            });
    }

    function convert(
        LibServerManager.Proof memory p
    ) internal pure returns (Proof memory) {
        return Proof({validity: convert(p.validity), context: p.context});
    }
}
