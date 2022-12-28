// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Authority Test
pragma solidity ^0.8.13;

import {TestBase} from "../../util/TestBase.sol";
import {Authority} from "contracts/consensus/authority/Authority.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {IHistory} from "contracts/history/IHistory.sol";
import {Vm} from "forge-std/Vm.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SimpleERC20} from "../../util/SimpleERC20.sol";

contract UntransferableToken is ERC20 {
    constructor(address minter, uint256 _initialSupply)
        ERC20("UntransferableToken", "UTFAB")
    {
        _mint(minter, _initialSupply);
    }

    function transfer(address, uint256) public pure override returns (bool) {
        return false;
    }
}

contract HistoryReverts is IHistory {
    function submitClaim(bytes calldata) external pure override {
        revert();
    }

    function migrateToConsensus(address) external pure override {
        revert();
    }

    function getEpochHash(address, bytes calldata)
        external
        pure
        override
        returns (
            bytes32,
            uint256,
            uint256
        )
    {
        revert();
    }
}

contract AuthorityTest is TestBase {
    Authority authority;

    // events
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
    event ConsensusCreated(address owner, IInputBox inputBox, IHistory history);
    event NewHistory(IHistory history);

    function testConstructor(
        address _owner,
        IInputBox _inputBox,
        IHistory _history
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        // two `OwnershipTransferred` events will be emitted during the constructor call
        // the first event is emitted by Ownable constructor
        // the second event is emitted by Authority constructor
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(this), _owner);
        // then the event `ConsensusCreated` will be emitted
        vm.expectEmit(false, false, false, true);
        emit ConsensusCreated(_owner, _inputBox, _history);

        authority = new Authority(_owner, _inputBox, _history);

        // check values set by constructor
        assertEq(authority.owner(), _owner);
        assertEq(address(authority.getHistory()), address(_history));
    }

    function testAuthorityConstructorOwner(
        IInputBox _inputBox,
        IHistory _history
    ) public {
        vm.recordLogs();
        authority = new Authority(address(this), _inputBox, _history);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        uint256 eventsFound;
        for (uint256 i; i < entries.length; ++i) {
            if (
                entries[i].topics[0] ==
                keccak256("OwnershipTransferred(address,address)")
            ) {
                assertEq(
                    entries[i].topics[1], // from
                    bytes32(uint256(uint160(address(0))))
                );
                assertEq(
                    entries[i].topics[2], // to
                    bytes32(uint256(uint160(address(this))))
                );
                eventsFound++;
            }
        }
        assertEq(eventsFound, 1);
    }

    function testRevertsOwnerAddressZero(IInputBox _inputBox, IHistory _history)
        public
    {
        vm.expectRevert("Ownable: new owner is the zero address");
        new Authority(address(0), _inputBox, _history);
    }

    function testMigrateHistory(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _newConsensus
    ) public isMockable(address(_history)) {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));
        vm.assume(_newConsensus != address(0));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
        vm.assume(address(_history) != address(authority));
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.migrateToConsensus.selector,
                _newConsensus
            ),
            ""
        );

        // will fail as not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        authority.migrateHistoryToConsensus(_newConsensus);

        // can only be called by owner
        vm.prank(_owner);
        authority.migrateHistoryToConsensus(_newConsensus);
    }

    function testSubmitClaim(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        bytes calldata _claim
    ) public isMockable(address(_history)) {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
        vm.assume(address(_history) != address(authority));
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(IHistory.submitClaim.selector),
            ""
        );

        // will fail as not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        authority.submitClaim(_claim);

        // can only be called by owner
        vm.prank(_owner);
        authority.submitClaim(_claim);
    }

    function testSetHistory(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        IHistory _newHistory
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox, _history);

        // before setting new history
        assertEq(address(authority.getHistory()), address(_history));

        // set new history
        // will fail as not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        authority.setHistory(_newHistory);
        // can only be called by owner
        vm.prank(_owner);
        // expect event NewHistory
        vm.expectEmit(false, false, false, true);
        emit NewHistory(_newHistory);
        authority.setHistory(_newHistory);

        // after setting new history
        assertEq(address(authority.getHistory()), address(_newHistory));
    }

    function testGetEpochHash(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _dapp,
        bytes calldata _claimProof,
        bytes32 _r0,
        uint256 _r1,
        uint256 _r2
    ) public isMockable(address(_history)) {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
        vm.assume(address(_history) != address(authority));
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.getEpochHash.selector,
                _dapp,
                _claimProof
            ),
            abi.encode(_r0, _r1, _r2)
        );

        // perform call
        (bytes32 r0, uint256 r1, uint256 r2) = authority.getEpochHash(
            _dapp,
            _claimProof
        );

        // check result
        assertEq(_r0, r0);
        assertEq(_r1, r1);
        assertEq(_r2, r2);
    }

    // test behaviors when history reverts
    function testHistoryReverts(
        address _owner,
        IInputBox _inputBox,
        address _dapp,
        bytes calldata _claim,
        address _consensus,
        bytes calldata _claimProof
    ) public {
        vm.assume(_owner != address(0));

        HistoryReverts historyR = new HistoryReverts();

        authority = new Authority(_owner, _inputBox, historyR);

        vm.expectRevert();
        vm.prank(_owner);
        authority.submitClaim(_claim);

        vm.expectRevert();
        vm.prank(_owner);
        authority.migrateHistoryToConsensus(_consensus);

        vm.expectRevert();
        authority.getEpochHash(_dapp, _claimProof);
    }

    function testWithdrawERC20TokensNotOwner(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _notOwner,
        IERC20 _token,
        address _recipient,
        uint256 _amount
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != _notOwner);

        authority = new Authority(_owner, _inputBox, _history);

        vm.prank(_notOwner);
        vm.expectRevert("Ownable: caller is not the owner");
        authority.withdrawERC20Tokens(_token, _recipient, _amount);
    }

    function testWithdrawERC20Tokens(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _recipient,
        uint256 _amount,
        uint256 _balance
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_recipient != address(0));
        vm.assume(_amount <= _balance);
        vm.assume(_balance < type(uint256).max);

        authority = new Authority(_owner, _inputBox, _history);

        vm.assume(_recipient != address(authority));

        // mint `_balance` ERC-20 tokens for authority contract
        IERC20 token = new SimpleERC20(address(authority), _balance);

        // try to transfer more than balance
        vm.prank(_owner);
        vm.expectRevert("ERC20: transfer amount exceeds balance");
        authority.withdrawERC20Tokens(token, _recipient, _balance + 1);

        // since transfer fails, all balances stay the same
        assertEq(token.balanceOf(address(authority)), _balance);
        assertEq(token.balanceOf(_recipient), 0);

        // it would succeed if the transfer amount is within balance
        vm.prank(_owner);
        authority.withdrawERC20Tokens(token, _recipient, _amount);

        // now check balance after a successful withdraw
        assertEq(token.balanceOf(address(authority)), _balance - _amount);
        assertEq(token.balanceOf(_recipient), _amount);
    }

    function testWithdrawERC20TokensFailed(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _recipient,
        uint256 _amount,
        uint256 _balance
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_recipient != address(0));
        vm.assume(_amount <= _balance);
        vm.assume(_balance < type(uint256).max);

        authority = new Authority(_owner, _inputBox, _history);

        vm.assume(_recipient != address(authority));

        // mint `_balance` ERC-20 tokens for authority contract
        IERC20 tokenFailed = new UntransferableToken(
            address(authority),
            _balance
        );

        // before failed withdraw
        assertEq(tokenFailed.balanceOf(address(authority)), _balance);
        assertEq(tokenFailed.balanceOf(_recipient), 0);

        // withdrawal fails because `transfer` returns `false`
        vm.prank(_owner);
        vm.expectRevert("Authority: withdrawal failed");
        authority.withdrawERC20Tokens(tokenFailed, _recipient, _amount);

        // after failed withdraw. All balances stay the same
        assertEq(tokenFailed.balanceOf(address(authority)), _balance);
        assertEq(tokenFailed.balanceOf(_recipient), 0);
    }
}
