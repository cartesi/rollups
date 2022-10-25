// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp Test
pragma solidity ^0.8.13;

import {TestBase} from "../TestBase.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {OutputValidityProof, LibOutputValidation} from "contracts/library/LibOutputValidation.sol";
import {SimpleERC20} from "./helper/SimpleERC20.sol";
import {SimpleERC721} from "./helper/SimpleERC721.sol";
import {SimpleERC721Receiver} from "./helper/SimpleERC721Receiver.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {IERC721Receiver} from "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import "forge-std/console.sol";

import {LibNoticeProof0} from "./helper/LibNoticeProof0.sol";
import {LibNoticeProof1} from "./helper/LibNoticeProof1.sol";
import {LibVoucherProof3} from "./helper/LibVoucherProof3.sol";
import {LibVoucherProof4} from "./helper/LibVoucherProof4.sol";
import {LibVoucherProof5} from "./helper/LibVoucherProof5.sol";

contract EtherReceiver {
    receive() external payable {}
}

contract CartesiDAppTest is TestBase {
    CartesiDApp dapp;
    IERC20 erc20Token;
    IERC721 erc721Token;
    IERC721Receiver erc721Receiver;
    OutputValidityProof proof;

    uint256 constant initialSupply = 1000000;
    uint256 constant transferAmount = 7;
    uint256 constant tokenId = uint256(keccak256("tokenId"));
    address constant dappOwner = address(bytes20(keccak256("dappOwner")));
    address constant tokenOwner = address(bytes20(keccak256("tokenOwner")));
    address constant recipient = address(bytes20(keccak256("recipient")));
    address constant consensus = address(bytes20(keccak256("consensus")));
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

    function testConstructorWithOwnerAsZeroAddress(
        IConsensus _consensus,
        bytes32 _templateHash
    ) public {
        vm.expectRevert("Ownable: new owner is the zero address");
        new CartesiDApp(_consensus, address(0), _templateHash);
    }

    function testConstructor(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash
    ) public {
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
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // check set values
        assertEq(address(dapp.getConsensus()), address(_consensus));
        assertEq(dapp.owner(), _owner);
        assertEq(dapp.getTemplateHash(), _templateHash);
    }

    function logVoucher(uint256 number, address destination, bytes memory payload) internal view {
        console.log("Hint: You might need to update the output validity proofs.");
        console.log("For more information, see `test/foundry/dapp/helper/README.md`.");
        console.log();
        console.log("Destination and payload of voucher %s:", number);
        console.log(destination);
        console.logBytes(payload);
    }

    function testExecuteVoucherAndEvent(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        logVoucher(3, address(erc20Token), erc20TransferPayload);

        // not able to execute voucher because dapp has 0 balance
        assertEq(erc20Token.balanceOf(address(dapp)), 0);
        assertEq(erc20Token.balanceOf(recipient), 0);
        bool success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
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
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
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

    function testRevertsReexecution(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        erc20Token.transfer(address(dapp), dappInitBalance);

        // 1st execution attempt should succeed
        bool success = dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );
        assertEq(success, true);

        // 2nd execution attempt should fail
        vm.expectRevert("re-execution not allowed");
        dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );

        // end result should be the same as executing successfully only once
        assertEq(
            erc20Token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(erc20Token.balanceOf(recipient), transferAmount);
    }

    function testRevertsEpochHash(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        proof.vouchersEpochRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert("incorrect epochHash");
        dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );
    }

    function testRevertsInputIndices(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        proof.epochInputIndex = 0xdeadbeef;

        vm.expectRevert("epoch input indices don't match");
        dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );
    }

    function testRevertsOutputsEpochRootHash(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        proof.outputHashesRootHash = bytes32(uint256(0xdeadbeef));

        vm.expectRevert("incorrect outputsEpochRootHash");
        dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );
    }

    function testRevertsOutputHashesRootHash(uint256 _inputIndex) public {
        setupForVoucher3(_inputIndex);

        proof.outputIndex = 0xdeadbeef;

        vm.expectRevert("incorrect outputHashesRootHash");
        dapp.executeVoucher(
            address(erc20Token),
            erc20TransferPayload,
            "",
            proof
        );
    }

    function setupForVoucher3(uint256 _inputIndex) internal {
        dapp = deployDAppDeterministically();
        erc20Token = deployERC20Deterministically();
        registerProof(_inputIndex, LibVoucherProof3.getProof());
    }

    // test ether transfer

    function testEtherTransfer(uint256 _inputIndex) public {
        dapp = deployDAppDeterministically();

        bytes memory withdrawEtherPayload = abi.encodeWithSelector(
            CartesiDApp.withdrawEther.selector,
            recipient,
            transferAmount
        );

        logVoucher(4, address(dapp), withdrawEtherPayload);

        registerProof(_inputIndex, LibVoucherProof4.getProof());

        // not able to execute voucher because dapp has 0 balance
        assertEq(address(dapp).balance, 0);
        assertEq(address(recipient).balance, 0);
        bool success = dapp.executeVoucher(
            address(dapp),
            withdrawEtherPayload,
            "",
            proof
        );
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
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(dapp),
            withdrawEtherPayload,
            "",
            proof
        );

        // check result
        assertEq(success, true);
        assertEq(address(dapp).balance, dappInitBalance - transferAmount);
        assertEq(address(recipient).balance, transferAmount);

        // cannot execute the same voucher again
        vm.expectRevert("re-execution not allowed");
        dapp.executeVoucher(address(dapp), withdrawEtherPayload, "", proof);
    }

    function testWithdrawEtherContract(uint256 _value, address _notDApp)
        public
    {
        dapp = deployDAppDeterministically();
        vm.assume(_value <= address(this).balance);
        vm.assume(_notDApp != address(dapp));
        address receiver = address(new EtherReceiver());

        // fund dapp
        vm.deal(address(dapp), _value);

        // withdrawEther cannot be called by anyone
        vm.expectRevert("only itself");
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
        dapp = deployDAppDeterministically();
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
        vm.expectRevert("only itself");
        vm.prank(_notDApp);
        dapp.withdrawEther(receiver, _value);

        // withdrawEther can only be called by dapp itself
        uint256 preBalance = receiver.balance;
        vm.prank(address(dapp));
        dapp.withdrawEther(receiver, _value);
        assertEq(receiver.balance, preBalance + _value);
        assertEq(address(dapp).balance, 0);
    }

    // test NFT transfer

    function testWithdrawNFT(uint256 _inputIndex) public {
        dapp = deployDAppDeterministically();
        erc721Token = deployERC721Deterministically();
        erc721Receiver = deployERC721ReceiverDeterministically();

        bytes memory safeTransferFromPayload = abi.encodeWithSignature(
            "safeTransferFrom(address,address,uint256)",
            dapp, // from
            erc721Receiver, // to
            tokenId
        );

        logVoucher(5, address(erc721Token), safeTransferFromPayload);

        registerProof(_inputIndex, LibVoucherProof5.getProof());

        // not able to execute voucher because dapp doesn't have the nft
        assertEq(erc721Token.ownerOf(tokenId), tokenOwner);
        bool success = dapp.executeVoucher(
            address(erc721Token),
            safeTransferFromPayload,
            "",
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
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        success = dapp.executeVoucher(
            address(erc721Token),
            safeTransferFromPayload,
            "",
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
            "",
            proof
        );
    }

    // test notices

    function testValidateNotice0(uint256 _inputIndex) public {
        dapp = deployDAppDeterministically();
        registerProof(_inputIndex, LibNoticeProof0.getProof());

        // validate notice 0
        bytes memory notice = abi.encodePacked(bytes4(0xdeadbeef));
        bool ret = dapp.validateNotice(notice, "", proof);
        assertEq(ret, true);

        // reverts if notice is incorrect
        bytes memory falseNotice = abi.encodePacked(bytes4(0xdeaddead));
        vm.expectRevert("incorrect outputHashesRootHash");
        dapp.validateNotice(falseNotice, "", proof);
    }

    function testValidateNotice1(uint256 _inputIndex) public {
        dapp = deployDAppDeterministically();
        registerProof(_inputIndex, LibNoticeProof1.getProof());

        // validate notice 1
        bytes memory notice = abi.encodePacked(bytes4(0xbeefdead));
        bool ret = dapp.validateNotice(notice, "", proof);
        assertEq(ret, true);
    }

    // test migration

    function testMigrateToConsensus(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        IConsensus _newConsensus,
        address _newOwner,
        address _nonZeroAddress
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));
        vm.assume(_owner != _newOwner);
        vm.assume(address(_newOwner) != address(0));
        vm.assume(_nonZeroAddress != address(0));

        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // migrate fail if not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        dapp.migrateToConsensus(_newConsensus);

        // now impersonate owner
        vm.prank(_owner);
        vm.expectEmit(false, false, false, true, address(dapp));
        emit NewConsensus(_newConsensus);
        dapp.migrateToConsensus(_newConsensus);
        assertEq(address(dapp.getConsensus()), address(_newConsensus));

        // if owner changes, then original owner no longer can migrate consensus
        vm.prank(_owner);
        dapp.transferOwnership(_newOwner);
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(_owner);
        dapp.migrateToConsensus(_consensus);

        // if new owner renounce ownership (give ownership to address 0)
        // no one will be able to migrate consensus
        vm.prank(_newOwner);
        dapp.renounceOwnership();
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(_nonZeroAddress);
        dapp.migrateToConsensus(_consensus);
    }

    // Stores `_proof` in storage variable `proof`
    // Mock `consensus` so that `getEpochHash` return values
    // can be used to validate `_proof` and `_inputIndex`.
    function registerProof(
        uint256 _inputIndex,
        OutputValidityProof memory _proof
    ) internal {
        // calculate epoch hash from proof
        bytes32 epochHash = keccak256(
            abi.encodePacked(
                _proof.vouchersEpochRootHash,
                _proof.noticesEpochRootHash,
                _proof.machineStateHash
            )
        );

        // mock the consensus contract to return the right epoch hash
        vm.mockCall(
            consensus,
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHash, _inputIndex, _proof.epochInputIndex)
        );

        // store proof in storage
        proof = _proof;
    }

    function deployDAppDeterministically() internal returns (CartesiDApp) {
        vm.prank(dappOwner);
        return
            new CartesiDApp{salt: salt}(
                IConsensus(consensus),
                dappOwner,
                templateHash
            );
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
}
