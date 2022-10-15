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
import {ICartesiDApp} from "contracts/dapp/ICartesiDApp.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {LibProof as LibVoucherProof3} from "./helper/voucherProof3.sol";
import {LibProof as LibVoucherProof4} from "./helper/voucherProof4.sol";
import {LibProof as LibVoucherProof5} from "./helper/voucherProof5.sol";
import {LibProof as LibNoticeProof0} from "./helper/noticeProof0.sol";
import {LibProof as LibNoticeProof1} from "./helper/noticeProof1.sol";
import {OutputValidityProof, LibOutputValidation} from "contracts/library/LibOutputValidation.sol";
import {SimpleToken} from "./helper/SimpleToken.sol";
import {SimpleNFT} from "./helper/SimpleNFT.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "forge-std/console.sol";

contract EtherReceiver {
    receive() external payable {}
}

contract CartesiDAppTest is TestBase {
    CartesiDApp dapp;
    SimpleToken token;
    OutputValidityProof proof;

    bool constant log_vouchers = false;
    uint256 constant initialSupply = 1000000;
    uint256 constant transferAmount = 7;
    address constant tokenOwner = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address constant recipient = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;
    bytes constant payload =
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

    function testConstructor(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash
    ) public {
        vm.assume(_owner != address(0));

        // 2 `OwnershipTransferred` events will be emitted during the constructor call
        // the first event is emitted by Ownable constructor
        // the second event is emitted by CartesiDApp constructor
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(this), _owner);

        // perform call to constructor
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // check set values
        assertEq(address(dapp.getConsensus()), address(_consensus));
        assertEq(dapp.owner(), _owner);
        assertEq(dapp.getTemplateHash(), _templateHash);
    }

    function testExecuteVoucherAndEvent(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        if (log_vouchers) {
            console.log("voucher 3:");
            console.log(address(token));
            console.logBytes(payload);
            revert("Transaction reverted on purpose to log debug info");
        }

        // not able to execute voucher because dapp has 0 balance
        assertEq(token.balanceOf(address(dapp)), 0);
        assertEq(token.balanceOf(recipient), 0);
        bool returnedVal = dapp.executeVoucher(
            address(token),
            payload,
            "",
            proof
        );
        assertEq(returnedVal, false);
        assertEq(token.balanceOf(address(dapp)), 0);
        assertEq(token.balanceOf(recipient), 0);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        token.transfer(address(dapp), dappInitBalance);
        assertEq(token.balanceOf(address(dapp)), dappInitBalance);
        assertEq(token.balanceOf(recipient), 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        returnedVal = dapp.executeVoucher(address(token), payload, "", proof);

        // check result
        assertEq(returnedVal, true);
        assertEq(
            token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(token.balanceOf(recipient), transferAmount);
    }

    function testRevertsReexecution(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        // fund dapp
        uint256 dappInitBalance = 100;
        vm.prank(tokenOwner);
        token.transfer(address(dapp), dappInitBalance);

        // 1st execution
        dapp.executeVoucher(address(token), payload, "", proof);

        // epect revert for re-execution
        vm.expectRevert("re-execution not allowed");
        // perform call
        dapp.executeVoucher(address(token), payload, "", proof);

        // result is the same as only 1 execution
        assertEq(
            token.balanceOf(address(dapp)),
            dappInitBalance - transferAmount
        );
        assertEq(token.balanceOf(recipient), transferAmount);
    }

    function testRevertsEpochHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public isMockable(address(_consensus)) {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        // epochHash incorrect
        bytes32 epochHashForVoucher = bytes32("wrong epoch hash");
        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForVoucher, _inputIndex, proof.epochInputIndex)
        );
        // epect revert
        vm.expectRevert("epochHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", proof);
    }

    function testRevertsInputIndices(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        // alter epoch input index
        proof.epochInputIndex += 1;
        // epect revert
        vm.expectRevert("epoch input indices don't match");
        // perform call
        dapp.executeVoucher(address(token), payload, "", proof);
    }

    function testRevertsOutputsEpochRootHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        // alter outputHashesRootHash
        proof.outputHashesRootHash = bytes32(0);
        // epect revert
        vm.expectRevert("outputsEpochRootHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", proof);
    }

    function testRevertsOutputHashesRootHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher3(_consensus, _owner, _templateHash, _inputIndex);

        // alter outputIndex
        proof.outputIndex += 1;
        // epect revert
        vm.expectRevert("outputHashesRootHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", proof);
    }

    function setupForVoucher3(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) internal isMockable(address(_consensus)) {
        vm.assume(_owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // deploy token contract deterministically
        vm.prank(tokenOwner);
        token = new SimpleToken{salt: bytes32(bytes20(tokenOwner))}(
            initialSupply
        );

        // get voucher proof from generated Solidity library
        proof = LibVoucherProof3.getProof();

        // epoch hash
        bytes32 epochHashForVoucher = keccak256(
            abi.encodePacked(
                proof.vouchersEpochRootHash,
                proof.noticesEpochRootHash,
                proof.machineStateHash
            )
        );

        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForVoucher, _inputIndex, proof.epochInputIndex)
        );
    }

    // test ether transfer

    function testEtherTransfer(uint256 _inputIndex) public {
        // set random addresses so that the deployment of CartesiDApp can be deterministic
        IConsensus consensus = IConsensus(
            0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
        );
        address dappOwner = 0x90F79bf6EB2c4f870365E785982E1f101E93b906;
        bytes32 templateHash = 0x08ca29d14db8627e335084ed01fe9da85598dde22ae7820fdfa67fe085a1e5e6;

        // deterministically deploy dapp from dappOwner
        vm.prank(dappOwner);
        CartesiDApp deterministicDapp = new CartesiDApp{
            salt: bytes32(bytes20(dappOwner))
        }(consensus, dappOwner, templateHash);
        bytes memory etherPayload = abi.encodeWithSelector(
            CartesiDApp.withdrawEther.selector,
            recipient,
            transferAmount
        );
        if (log_vouchers) {
            console.log("voucher 4:");
            console.log(address(deterministicDapp)); // changes when CartesiDApp bytecode changes
            console.logBytes(etherPayload);
            revert("Transaction reverted on purpose to log debug info");
        }

        // get voucher proof from generated Solidity library
        proof = LibVoucherProof4.getProof();

        // epoch hash
        bytes32 epochHashForVoucher = keccak256(
            abi.encodePacked(
                proof.vouchersEpochRootHash,
                proof.noticesEpochRootHash,
                proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForVoucher, _inputIndex, proof.epochInputIndex)
        );

        // not able to execute voucher because deterministicDapp has 0 balance
        assertEq(address(deterministicDapp).balance, 0);
        assertEq(address(recipient).balance, 0);
        bool returnedVal = deterministicDapp.executeVoucher(
            address(deterministicDapp),
            etherPayload,
            "",
            proof
        );
        assertEq(returnedVal, false);
        assertEq(address(deterministicDapp).balance, 0);
        assertEq(address(recipient).balance, 0);

        // fund deterministicDapp
        uint256 dappInitBalance = 100;
        (bool sent, ) = address(deterministicDapp).call{value: dappInitBalance}(
            ""
        );
        require(sent, "fail to fund deterministicDapp");
        assertEq(address(deterministicDapp).balance, dappInitBalance);
        assertEq(address(recipient).balance, 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(deterministicDapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        returnedVal = deterministicDapp.executeVoucher(
            address(deterministicDapp),
            etherPayload,
            "",
            proof
        );

        // check result
        assertEq(returnedVal, true);
        assertEq(
            address(deterministicDapp).balance,
            dappInitBalance - transferAmount
        );
        assertEq(address(recipient).balance, transferAmount);

        // cannot execute the same voucher again
        vm.expectRevert("re-execution not allowed");
        deterministicDapp.executeVoucher(
            address(deterministicDapp),
            etherPayload,
            "",
            proof
        );
    }

    function testWithdrawEtherContract(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _value
    ) public {
        vm.assume(_owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);
        vm.assume(_owner != address(dapp));
        vm.assume(_value <= address(this).balance);
        address receiver = address(new EtherReceiver());

        // fund dapp
        (bool sent, ) = address(dapp).call{value: _value}("");
        require(sent, "fail to fund dapp");

        // withdrawEther cannot be called by `this`
        vm.expectRevert("only itself");
        dapp.withdrawEther(receiver, _value);

        // withdrawEther cannot be called by any address not equal to dapp address
        vm.expectRevert("only itself");
        vm.prank(_owner);
        dapp.withdrawEther(receiver, _value);

        // can only be called by dapp itself
        uint256 preBalance = receiver.balance;
        vm.prank(address(dapp));
        dapp.withdrawEther(receiver, _value);
        assertEq(receiver.balance, preBalance + _value);
    }

    function testWithdrawEtherEOA(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _receiverSeed,
        uint256 _value
    ) public {
        vm.assume(_owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);
        vm.assume(_owner != address(dapp));
        vm.assume(_value <= address(this).balance);
        address receiver = address(
            bytes20(keccak256(abi.encode(_receiverSeed)))
        );

        // assume receiver is not a contract
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(receiver)
        }
        vm.assume(codeSize == 0);

        // fund dapp
        (bool sent, ) = address(dapp).call{value: _value}("");
        require(sent, "fail to fund dapp");

        // withdrawEther cannot be called by `this`
        vm.expectRevert("only itself");
        dapp.withdrawEther(receiver, _value);

        // withdrawEther cannot be called by any address not equal to dapp address
        vm.expectRevert("only itself");
        vm.prank(_owner);
        dapp.withdrawEther(receiver, _value);

        // can only be called by dapp itself
        uint256 preBalance = receiver.balance;
        vm.prank(address(dapp));
        dapp.withdrawEther(receiver, _value);
        assertEq(receiver.balance, preBalance + _value);
    }

    // test NFT transfer

    function testWithdrawNFT(uint256 _inputIndex) public {
        // deterministically deploy dapp from tokenOwner
        vm.prank(tokenOwner);
        SimpleNFT snft = new SimpleNFT{salt: bytes32(bytes20(tokenOwner))}();

        // set random addresses so that the deployment of CartesiDApp can be deterministic
        IConsensus consensus = IConsensus(
            0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
        );
        address dappOwner = 0x90F79bf6EB2c4f870365E785982E1f101E93b906;
        bytes32 templateHash = 0x08ca29d14db8627e335084ed01fe9da85598dde22ae7820fdfa67fe085a1e5e6;
        // deterministically deploy dapp from dappOwner
        vm.prank(dappOwner);
        CartesiDApp deterministicDapp = new CartesiDApp{
            salt: bytes32(bytes20(dappOwner))
        }(consensus, dappOwner, templateHash);
        bytes memory NFTPayload = abi.encodeWithSelector(
            IERC721.transferFrom.selector,
            deterministicDapp, // from
            recipient, // to
            0 // tokenId
        );
        if (log_vouchers) {
            console.log("voucher 5:");
            console.log(address(snft));
            console.logBytes(NFTPayload); // changes when CartesiDApp bytecode changes
            revert("Transaction reverted on purpose to log debug info");
        }

        // get voucher proof from generated Solidity library
        proof = LibVoucherProof5.getProof();

        // epoch hash
        bytes32 epochHashForVoucher = keccak256(
            abi.encodePacked(
                proof.vouchersEpochRootHash,
                proof.noticesEpochRootHash,
                proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForVoucher, _inputIndex, proof.epochInputIndex)
        );

        // not able to execute voucher because deterministicDapp doesn't have the nft
        assertEq(snft.balanceOf(address(deterministicDapp)), 0);
        assertEq(snft.balanceOf(address(recipient)), 0);
        bool returnedVal = deterministicDapp.executeVoucher(
            address(snft),
            NFTPayload,
            "",
            proof
        );
        assertEq(returnedVal, false);
        assertEq(snft.balanceOf(address(deterministicDapp)), 0);
        assertEq(snft.balanceOf(address(recipient)), 0);

        // fund deterministicDapp
        vm.prank(tokenOwner);
        snft.transferFrom(tokenOwner, address(deterministicDapp), 0);
        assertEq(snft.balanceOf(address(deterministicDapp)), 1);
        assertEq(snft.balanceOf(address(recipient)), 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(deterministicDapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                proof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        returnedVal = deterministicDapp.executeVoucher(
            address(snft),
            NFTPayload,
            "",
            proof
        );

        // check result
        assertEq(returnedVal, true);
        assertEq(snft.balanceOf(address(deterministicDapp)), 0);
        assertEq(snft.balanceOf(address(recipient)), 1);
        assertEq(snft.ownerOf(0), address(recipient));

        // cannot execute the same voucher again
        vm.expectRevert("re-execution not allowed");
        deterministicDapp.executeVoucher(address(snft), NFTPayload, "", proof);
    }

    // test notices

    function testValidateNotice0(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public isMockable(address(_consensus)) {
        // *** setup for notice0 ***
        vm.assume(_owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // get notice proof from generated Solidity library
        proof = LibNoticeProof0.getProof();

        // epoch hash
        bytes32 epochHashForNotice = keccak256(
            abi.encodePacked(
                proof.vouchersEpochRootHash,
                proof.noticesEpochRootHash,
                proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForNotice, _inputIndex, proof.epochInputIndex)
        );
        // *** finish setup ***

        // validate notice0
        bytes memory notice = abi.encodePacked(bytes4(0xdeadbeef));
        bool returnedVal = dapp.validateNotice(notice, "", proof);
        assertEq(returnedVal, true);

        // reverts if notice is incorrect
        bytes memory falseNotice = abi.encodePacked(bytes4(0xdeaddead));
        vm.expectRevert("outputHashesRootHash incorrect");
        dapp.validateNotice(falseNotice, "", proof);
    }

    function testValidateNotice1(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public isMockable(address(_consensus)) {
        // *** setup for notice1 ***
        vm.assume(_owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // get notice proof from generated Solidity library
        proof = LibNoticeProof1.getProof();

        // epoch hash
        bytes32 epochHashForNotice = keccak256(
            abi.encodePacked(
                proof.vouchersEpochRootHash,
                proof.noticesEpochRootHash,
                proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(epochHashForNotice, _inputIndex, proof.epochInputIndex)
        );
        // *** finish setup ***

        // validate notice1
        bytes memory notice = abi.encodePacked(bytes4(0xbeefdead));
        bool returnedVal = dapp.validateNotice(notice, "", proof);
        assertEq(returnedVal, true);
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
        if (_newOwner != _owner) {
            vm.expectRevert("Ownable: caller is not the owner");
            vm.prank(_owner);
            dapp.migrateToConsensus(_consensus);
        }

        // if new owner renounce ownership (give ownership to address 0)
        // no one will be able to migrate consensus
        vm.prank(_newOwner);
        dapp.renounceOwnership();
        vm.expectRevert("Ownable: caller is not the owner");
        vm.prank(_nonZeroAddress);
        dapp.migrateToConsensus(_consensus);
    }
}
