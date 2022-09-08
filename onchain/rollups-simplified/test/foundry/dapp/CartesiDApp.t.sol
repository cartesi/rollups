// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {CartesiDApp} from "contracts/dapp/CartesiDApp.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {Proof as VoucherProofSol} from "./helper/voucherProof3.sol";
import {Proof as NoticeProofSol0} from "./helper/noticeProof0.sol";
import {Proof as NoticeProofSol1} from "./helper/noticeProof1.sol";
import {OutputValidityProof, LibOutputValidation} from "contracts/library/LibOutputValidation.sol";
import {SimpleToken} from "./helper/SimpleToken.sol";
import "forge-std/console.sol";

contract CartesiDAppTest is Test {
    CartesiDApp dapp;
    SimpleToken token;
    VoucherProofSol pSol; // auto-generated solidity contract version of proof
    OutputValidityProof voucherProof; // copy proof from contract to storage

    event VoucherExecuted(uint256 voucherPosition);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
    event NewConsensus(IConsensus newConsensus);

    uint256 initialSupply = 1000000;
    uint256 transfer_amount = 7;
    address tokenOwner = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address recipient = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;
    bytes payload =
        abi.encodeWithSelector(
            bytes4(keccak256("transfer(address,uint256)")),
            recipient,
            transfer_amount
        );

    function setUp() public {
        vm.prank(tokenOwner);
        // deterministic deployment
        token = new SimpleToken{salt: bytes32(bytes20(tokenOwner))}(
            initialSupply
        );
        // console.log(address(token));
        // console.logBytes(payload);
        pSol = new VoucherProofSol();
    }

    function testConstructor(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash
    ) public {
        vm.assume(address(_consensus) != address(0) && _owner != address(0));

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
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // not able to execute voucher because dapp has 0 balance
        assertEq(token.balanceOf(address(dapp)), 0);
        bool returnedVal = dapp.executeVoucher(
            address(token),
            payload,
            "",
            voucherProof
        );
        assertEq(returnedVal, false);
        assertEq(token.balanceOf(recipient), 0);

        // fund dapp
        uint256 dapp_init_balance = 100;
        vm.prank(tokenOwner);
        token.transfer(address(dapp), dapp_init_balance);
        assertEq(token.balanceOf(address(dapp)), dapp_init_balance);
        assertEq(token.balanceOf(recipient), 0);

        // expect event
        vm.expectEmit(false, false, false, true, address(dapp));
        emit VoucherExecuted(
            LibOutputValidation.getBitMaskPosition(
                voucherProof.outputIndex,
                _inputIndex
            )
        );

        // perform call
        returnedVal = dapp.executeVoucher(
            address(token),
            payload,
            "",
            voucherProof
        );

        // check result
        assertEq(
            token.balanceOf(address(dapp)),
            dapp_init_balance - transfer_amount
        );
        assertEq(token.balanceOf(recipient), transfer_amount);
        assertEq(returnedVal, true);
    }

    function testRevertsReexecution(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // fund dapp
        uint256 dapp_init_balance = 100;
        vm.prank(tokenOwner);
        token.transfer(address(dapp), dapp_init_balance);

        // 1st execution
        dapp.executeVoucher(address(token), payload, "", voucherProof);

        // epect revert for re-execution
        vm.expectRevert("re-execution not allowed");
        // perform call
        dapp.executeVoucher(address(token), payload, "", voucherProof);

        // result is the same as only 1 execution
        assertEq(
            token.balanceOf(address(dapp)),
            dapp_init_balance - transfer_amount
        );
        assertEq(token.balanceOf(recipient), transfer_amount);
    }

    function testRevertsEpochHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // epochHash incorrect
        bytes32 epochHashForVoucher = bytes32("wrong epoch hash");
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(
                epochHashForVoucher,
                _inputIndex,
                voucherProof.epochInputIndex
            )
        );
        // epect revert
        vm.expectRevert("epochHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", voucherProof);
    }

    function testRevertsInputIndices(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // alter epoch input index
        voucherProof.epochInputIndex += 1;
        // epect revert
        vm.expectRevert("epoch input indices don't match");
        // perform call
        dapp.executeVoucher(address(token), payload, "", voucherProof);
    }

    function testRevertsOutputsEpochRootHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // alter outputHashesRootHash
        voucherProof.outputHashesRootHash = bytes32(0);
        // epect revert
        vm.expectRevert("outputsEpochRootHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", voucherProof);
    }

    function testRevertsOutputHashesRootHash(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        setupForVoucher(_consensus, _owner, _templateHash, _inputIndex);

        // alter outputIndex
        voucherProof.outputIndex += 1;
        // epect revert
        vm.expectRevert("outputHashesRootHash incorrect");
        // perform call
        dapp.executeVoucher(address(token), payload, "", voucherProof);
    }

    function setupForVoucher(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) internal {
        vm.assume(address(_consensus) != address(0) && _owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);

        // copy proof from contract to storage
        (
            voucherProof.epochInputIndex,
            voucherProof.outputIndex,
            voucherProof.outputHashesRootHash,
            voucherProof.vouchersEpochRootHash,
            voucherProof.noticesEpochRootHash,
            voucherProof.machineStateHash
        ) = pSol.proof();
        voucherProof.keccakInHashesSiblings = pSol.getArray1();
        voucherProof.outputHashesInEpochSiblings = pSol.getArray2();

        // epoch hash
        bytes32 epochHashForVoucher = keccak256(
            abi.encode(
                voucherProof.vouchersEpochRootHash,
                voucherProof.noticesEpochRootHash,
                voucherProof.machineStateHash
            )
        );

        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(
                epochHashForVoucher,
                _inputIndex,
                voucherProof.epochInputIndex
            )
        );
    }

    // test notices

    function testValidateNotice0(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        // *** setup for notice0 ***
        vm.assume(address(_consensus) != address(0) && _owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);
        OutputValidityProof memory notice0Proof;
        NoticeProofSol0 nSol0 = new NoticeProofSol0();
        // proof
        (
            notice0Proof.epochInputIndex,
            notice0Proof.outputIndex,
            notice0Proof.outputHashesRootHash,
            notice0Proof.vouchersEpochRootHash,
            notice0Proof.noticesEpochRootHash,
            notice0Proof.machineStateHash
        ) = nSol0.proof();
        notice0Proof.keccakInHashesSiblings = nSol0.getArray1();
        notice0Proof.outputHashesInEpochSiblings = nSol0.getArray2();
        // epoch hash
        bytes32 epochHashForNotice = keccak256(
            abi.encode(
                notice0Proof.vouchersEpochRootHash,
                notice0Proof.noticesEpochRootHash,
                notice0Proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(
                epochHashForNotice,
                _inputIndex,
                notice0Proof.epochInputIndex
            )
        );
        // *** finish setup ***

        // validate notice0
        bytes memory notice = abi.encodePacked(bytes4(0xdeadbeef));
        bool returnedVal = dapp.validateNotice(notice, "", notice0Proof);
        assertEq(returnedVal, true);

        // reverts if notice is incorrect
        bytes memory falseNotice = abi.encodePacked(bytes4(0xdeaddead));
        vm.expectRevert("outputHashesRootHash incorrect");
        dapp.validateNotice(falseNotice, "", notice0Proof);
    }

    function testValidateNotice1(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        uint256 _inputIndex
    ) public {
        // *** setup for notice1 ***
        vm.assume(address(_consensus) != address(0) && _owner != address(0));
        dapp = new CartesiDApp(_consensus, _owner, _templateHash);
        OutputValidityProof memory notice1Proof;
        NoticeProofSol1 nSol1 = new NoticeProofSol1();
        // proof
        (
            notice1Proof.epochInputIndex,
            notice1Proof.outputIndex,
            notice1Proof.outputHashesRootHash,
            notice1Proof.vouchersEpochRootHash,
            notice1Proof.noticesEpochRootHash,
            notice1Proof.machineStateHash
        ) = nSol1.proof();
        notice1Proof.keccakInHashesSiblings = nSol1.getArray1();
        notice1Proof.outputHashesInEpochSiblings = nSol1.getArray2();
        // epoch hash
        bytes32 epochHashForNotice = keccak256(
            abi.encode(
                notice1Proof.vouchersEpochRootHash,
                notice1Proof.noticesEpochRootHash,
                notice1Proof.machineStateHash
            )
        );
        // mocking consensus
        vm.mockCall(
            address(_consensus),
            abi.encodeWithSelector(IConsensus.getEpochHash.selector),
            abi.encode(
                epochHashForNotice,
                _inputIndex,
                notice1Proof.epochInputIndex
            )
        );
        // *** finish setup ***

        // validate notice1
        bytes memory notice = abi.encodePacked(bytes4(0xbeefdead));
        bool returnedVal = dapp.validateNotice(notice, "", notice1Proof);
        assertEq(returnedVal, true);
    }

    // test migration
    function testMigrateToConsensus(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash,
        IConsensus _newConsensus,
        address _newOwner
    ) public {
        vm.assume(
            address(_consensus) != address(0) &&
                _owner != address(0) &&
                address(_newConsensus) != address(0) &&
                address(_newOwner) != address(0) &&
                address(_newOwner) != address(_owner)
        );
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
    }
}
