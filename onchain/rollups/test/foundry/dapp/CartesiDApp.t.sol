// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

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

import {SimpleConsensus} from "../util/SimpleConsensus.sol";
import {SimpleERC20} from "../util/SimpleERC20.sol";
import {SimpleERC721} from "../util/SimpleERC721.sol";
import {SimpleERC721Receiver} from "../util/SimpleERC721Receiver.sol";
import {SimpleContract} from "../util/SimpleContract.sol";

import {LibOutputProof0} from "./helper/LibOutputProof0.sol";
import {LibOutputProof1} from "./helper/LibOutputProof1.sol";
import {LibOutputProof2} from "./helper/LibOutputProof2.sol";
import {LibOutputProof3} from "./helper/LibOutputProof3.sol";
import {LibOutputProof4} from "./helper/LibOutputProof4.sol";
import {LibOutputProof5} from "./helper/LibOutputProof5.sol";
import {LibOutputProof6} from "./helper/LibOutputProof6.sol";
import {LibOutputProof7} from "./helper/LibOutputProof7.sol";

import "forge-std/console.sol";

// Outputs
// 0: notice 0xfafafafa
// 1: voucher ERC-20 transfer
// 2: voucher ERC-721 transfer
// 3: voucher ETH withdrawal to EOA without payload
// 4: voucher ETH withdrawal to EOA with payload
// 5: voucher ETH withdrawal to contract without payload
// 6: voucher ETH withdrawal to contract with invalid payload
// 7: voucher ETH withdrawal to contract with valid payload

contract CartesiDAppTest is TestBase {
    Proof proof;
    CartesiDApp dapp;
    IConsensus consensus;
    IERC20 erc20Token;
    IERC721 erc721Token;
    IERC721Receiver erc721Receiver;

    uint256 constant initialSupply = 1000000;
    uint256 constant transferAmount = 7;
    uint256 constant tokenId = uint256(keccak256("tokenId"));
    address constant dappOwner = address(bytes20(keccak256("dappOwner")));
    address constant tokenOwner = address(bytes20(keccak256("tokenOwner")));
    address constant recipient = address(bytes20(keccak256("recipient")));
    address constant noticeSender = address(bytes20(keccak256("noticeSender")));
    bytes32 constant salt = keccak256("salt");
    bytes32 constant templateHash = keccak256("templateHash");

    bytes constant erc20TransferPayload =
        abi.encodeWithSelector(
            IERC20.transfer.selector,
            recipient,
            transferAmount
        );

    event VoucherExecuted(uint256 voucherPosition);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
    event NewConsensus(IConsensus newConsensus);

    function setUp() public {
        consensus = deployConsensusDeterministically();
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

    function logInput(
        uint256 number,
        address sender,
        bytes memory payload
    ) internal view {
        console.log("Proof for output %d might be outdated.", number);
        console.log(sender);
        console.logBytes(payload);
        console.log("For more info, see `test/foundry/dapp/helper/README.md`.");
    }

    function logVoucher(
        uint256 number,
        address destination,
        bytes memory payload
    ) internal view {
        logVoucher(number, destination, 0, payload);
    }

    function logVoucher(
        uint256 number,
        address destination,
        uint256 value,
        bytes memory payload
    ) internal view {
        bytes memory output = abi.encodePacked(destination, value, payload);
        logInput(number, noticeSender, output);
    }

    function logNotice(uint256 number, bytes memory notice) internal view {
        bytes memory output = abi.encodePacked(notice);
        logInput(number, noticeSender, output);
    }

    // test notices

    function testNoticeValidation(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically();
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof0.getNoticeProof()
        );

        bytes memory notice = abi.encodePacked(bytes4(0xfafafafa));
        logNotice(0, notice);
        bool ret = dapp.validateNotice(notice, proof);
        assertEq(ret, true);

        // reverts if notice is incorrect
        bytes memory falseNotice = abi.encodePacked(bytes4(0xdeaddead));
        vm.expectRevert("incorrect outputHashesRootHash");
        dapp.validateNotice(falseNotice, proof);
    }

    // test vouchers

    function testExecuteVoucherAndEvent(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        logVoucher(1, address(erc20Token), erc20TransferPayload);

        // not able to execute voucher because dapp has 0 balance
        assertEq(erc20Token.balanceOf(address(dapp)), 0);
        assertEq(erc20Token.balanceOf(recipient), 0);
        bool success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            proof
        );
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
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(
            erc20Token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(erc20Token.balanceOf(recipient), transferAmount);
    }

    function testRevertsReexecution(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);

        // 1st execution attempt should succeed
        bool success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            proof
        );
        assertEq(success, true);

        // 2nd execution attempt should fail
        vm.expectRevert("re-execution not allowed");
        dapp.executeVoucher(address(erc20Token), erc20TransferPayload, proof);

        // end result should be the same as executing successfully only once
        assertEq(
            erc20Token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(erc20Token.balanceOf(recipient), transferAmount);
    }

    // `_inboxInputIndex` and `_outputIndex` are always less than 2**128
    function testWasVoucherExecutedForAny(
        uint128 _inboxInputIndex,
        uint128 _numInputsAfter,
        uint128 _outputIndex
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        bool executed = dapp.wasVoucherExecuted(_inboxInputIndex, _outputIndex);
        assertEq(executed, false);
    }

    function testWasVoucherExecuted(
        uint128 _inboxInputIndex,
        uint128 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        // before executing voucher
        bool executed = dapp.wasVoucherExecuted(
            _inboxInputIndex,
            proof.validity.outputIndex
        );
        assertEq(executed, false);

        // execute voucher - failed
        bool success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            proof
        );
        assertEq(success, false);

        // `wasVoucherExecuted` should still return false
        executed = dapp.wasVoucherExecuted(
            _inboxInputIndex,
            proof.validity.outputIndex
        );
        assertEq(executed, false);

        // execute voucher - succeeded
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);
        success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            proof
        );
        assertEq(success, true);

        // after executing voucher, `wasVoucherExecuted` should return true
        executed = dapp.wasVoucherExecuted(
            _inboxInputIndex,
            proof.validity.outputIndex
        );
        assertEq(executed, true);
    }

    function testRevertsEpochHash(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        proof.validity.vouchersEpochRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert("incorrect epochHash");
        dapp.executeVoucher(address(erc20Token), erc20TransferPayload, proof);
    }

    function testRevertsOutputsEpochRootHash(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        proof.validity.outputHashesRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert("incorrect outputsEpochRootHash");
        dapp.executeVoucher(address(erc20Token), erc20TransferPayload, proof);
    }

    function testRevertsOutputHashesRootHash(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        setupERC20TransferVoucher(_inboxInputIndex, _numInputsAfter);

        proof.validity.outputIndex = 0xdeadbeef;

        vm.expectRevert("incorrect outputHashesRootHash");
        dapp.executeVoucher(address(erc20Token), erc20TransferPayload, proof);
    }

    function testRevertsInputIndexOOB(uint256 _inboxInputIndex) public {
        setupERC20TransferVoucher(_inboxInputIndex, 0);

        // If the epoch input index were 0, then there would be no way for the
        // inbox input index to be out of bounds because every claim is non-empty,
        // as it must contain at least one input
        assert(proof.validity.inputIndex > 0);

        // This assumption aims to avoid an integer overflow in the CartesiDApp
        vm.assume(
            _inboxInputIndex <= type(uint256).max - proof.validity.inputIndex
        );

        // Calculate epoch hash from proof
        bytes32 epochHash = calculateEpochHash(proof.validity);

        // Mock consensus again to return a claim that spans only 1 input,
        // but we are registering a proof whose epoch input index is 1...
        // so the proof would succeed but the input would be out of bounds
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getClaim.selector),
            abi.encode(epochHash, _inboxInputIndex, _inboxInputIndex)
        );

        vm.expectRevert("inbox input index out of bounds");
        dapp.executeVoucher(address(erc20Token), erc20TransferPayload, proof);
    }

    function setupERC20TransferVoucher(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) internal {
        dapp = deployDAppDeterministically();
        erc20Token = deployERC20Deterministically();
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof1.getVoucherProof()
        );
    }

    // test NFT transfer

    function testWithdrawNFT(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically();
        erc721Token = deployERC721Deterministically();
        erc721Receiver = deployERC721ReceiverDeterministically();

        bytes memory safeTransferFromPayload = abi.encodeWithSignature(
            "safeTransferFrom(address,address,uint256)",
            dapp, // from
            erc721Receiver, // to
            tokenId
        );

        logVoucher(2, address(erc721Token), safeTransferFromPayload);

        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof2.getVoucherProof()
        );

        // not able to execute voucher because dapp doesn't have the nft
        assertEq(erc721Token.ownerOf(tokenId), tokenOwner);
        bool success = dapp.executeVoucher(
            address(erc721Token),
            safeTransferFromPayload,
            proof
        );
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
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(erc721Token),
            safeTransferFromPayload,
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(erc721Token.ownerOf(tokenId), address(erc721Receiver));

        // cannot execute the same voucher again
        vm.expectRevert("re-execution not allowed");
        dapp.executeVoucher(
            address(erc721Token),
            safeTransferFromPayload,
            proof
        );
    }

    // test ether transfer

    function testWithdrawEtherToEOANoPayload(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically(); // doesn't have to be deterministic here
        address recipient_asm = recipient; // for the use of assembly
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(recipient_asm)
        }
        require(codeSize == 0, "recipient is not EOA");

        logVoucher(3, recipient, transferAmount, "");
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof3.getVoucherProof()
        );

        // not able to execute voucher because dapp has 0 balance
        assertEq(address(dapp).balance, 0);
        assertEq(recipient.balance, 0);
        vm.expectCall(recipient, transferAmount, "");
        bool success = dapp.executeVoucher(
            recipient,
            transferAmount,
            "",
            proof
        );
        assertEq(success, false);
        assertEq(address(dapp).balance, 0);
        assertEq(recipient.balance, 0);

        // fund dapp
        uint256 dappInitBalance = transferAmount;
        vm.deal(address(dapp), dappInitBalance);
        assertEq(address(dapp).balance, dappInitBalance);
        assertEq(recipient.balance, 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(recipient, transferAmount, "", proof);

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(recipient.balance, transferAmount);

        // cannot execute the same voucher again
        vm.expectRevert("re-execution not allowed");
        dapp.executeVoucher(recipient, transferAmount, "", proof);
    }

    function testWithdrawEtherToEOAWithPayload(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically(); // doesn't have to be deterministic here

        // fund dapp
        uint256 dappInitBalance = transferAmount;
        vm.deal(address(dapp), dappInitBalance);

        bytes memory randomPayload = abi.encodeWithSignature(
            "randomFunction()"
        );
        logVoucher(4, recipient, transferAmount, randomPayload);
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof4.getVoucherProof()
        );

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        bool success = dapp.executeVoucher(
            recipient,
            transferAmount,
            randomPayload,
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(recipient.balance, transferAmount);
    }

    function testWithdrawEtherToContractNoPayload(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically(); // doesn't have to be deterministic here
        SimpleContract simpleContract = deploySimpleContractDeterministically();

        // fund dapp
        uint256 dappInitBalance = transferAmount;
        vm.deal(address(dapp), dappInitBalance);

        logVoucher(5, address(simpleContract), transferAmount, "");
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof5.getVoucherProof()
        );

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        bool success = dapp.executeVoucher(
            address(simpleContract),
            transferAmount,
            "",
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(address(simpleContract).balance, transferAmount);
    }

    function testWithdrawEtherToContractWithPayload(
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter
    ) public {
        dapp = deployDAppDeterministically(); // doesn't have to be deterministic here
        SimpleContract simpleContract = deploySimpleContractDeterministically();

        // fund dapp
        uint256 dappInitBalance = transferAmount;
        vm.deal(address(dapp), dappInitBalance);

        bytes memory randomPayload = abi.encodeWithSignature(
            "randomFunction()"
        );
        logVoucher(6, address(simpleContract), transferAmount, randomPayload);
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof6.getVoucherProof()
        );

        // perform a failed call
        vm.expectCall(address(simpleContract), transferAmount, randomPayload);
        bool success = dapp.executeVoucher(
            address(simpleContract),
            transferAmount,
            randomPayload,
            proof
        );
        assertEq(success, false);
        assertEq(address(dapp).balance, dappInitBalance);
        assertEq(address(simpleContract).balance, 0);

        // now to call a real existing function
        bytes memory payload = abi.encodeWithSignature("depositEther()");
        logVoucher(7, address(simpleContract), transferAmount, payload);
        registerProof(
            _inboxInputIndex,
            _numInputsAfter,
            LibOutputProof7.getVoucherProof()
        );

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.validity.outputIndex,
                _inboxInputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(simpleContract),
            transferAmount,
            payload,
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(address(simpleContract).balance, transferAmount);
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
        uint256 _inboxInputIndex,
        uint256 _numInputsAfter,
        OutputValidityProof memory _validity
    ) internal {
        // check if `_inboxInputIndex` and `_numInputsAfter` are valid
        vm.assume(_validity.inputIndex <= _inboxInputIndex);
        vm.assume(_numInputsAfter <= type(uint256).max - _inboxInputIndex);

        // calculate epoch hash from proof
        bytes32 epochHash = calculateEpochHash(_validity);

        // calculate input index range based on proof and fuzzy variables
        uint256 firstInputIndex = _inboxInputIndex - _validity.inputIndex;
        uint256 lastInputIndex = _inboxInputIndex + _numInputsAfter;

        // mock the consensus contract to return the right epoch hash
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getClaim.selector),
            abi.encode(epochHash, firstInputIndex, lastInputIndex)
        );

        // store proof in storage
        proof = Proof({validity: _validity, context: ""});
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

    function deploySimpleContractDeterministically()
        internal
        returns (SimpleContract)
    {
        vm.prank(tokenOwner);
        return new SimpleContract{salt: salt}();
    }

    function calculateEpochHash(
        OutputValidityProof memory _validity
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    _validity.vouchersEpochRootHash,
                    _validity.noticesEpochRootHash,
                    _validity.machineStateHash
                )
            );
    }
}
