// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Authority Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {Authority} from "contracts/consensus/authority/Authority.sol";
import {IConsensus} from "contracts/consensus/IConsensus.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {IHistory} from "contracts/history/IHistory.sol";

contract AuthorityTest is Test {
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
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));
        vm.assume(_newConsensus != address(0));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
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
        address _dapp,
        bytes calldata _claim
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.submitClaim.selector,
                _dapp,
                _claim
            ),
            ""
        );

        // will fail as not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        authority.submitClaim(_dapp, _claim);

        // can only be called by owner
        vm.prank(_owner);
        authority.submitClaim(_dapp, _claim);
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
    ) public {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox, _history);

        // mocking history
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
}
