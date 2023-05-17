// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Authority Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {TestBase} from "../../util/TestBase.sol";
import {Authority, AuthorityWithdrawalFailed} from "contracts/consensus/authority/Authority.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {IHistory} from "contracts/history/IHistory.sol";
import {History} from "contracts/history/History.sol";
import {Vm} from "forge-std/Vm.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SimpleERC20} from "../../util/SimpleERC20.sol";

contract UntransferableToken is ERC20 {
    constructor(
        address minter,
        uint256 _initialSupply
    ) ERC20("UntransferableToken", "UTFAB") {
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

    function getClaim(
        address,
        bytes calldata
    ) external pure override returns (bytes32, uint256, uint256) {
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
    event ConsensusCreated(address owner, IInputBox inputBox);
    event NewHistory(IHistory history);
    event ApplicationJoined(address application);

    function testConstructor(address _owner, IInputBox _inputBox) public {
        vm.assume(_owner != address(0));
        uint256 numOfEvents;

        // two `OwnershipTransferred` events might be emitted during the constructor call
        // the first event is emitted by Ownable constructor
        vm.expectEmit(true, true, false, false);
        emit OwnershipTransferred(address(0), address(this));
        ++numOfEvents;

        // a second event is emitted by Authority constructor iff msg.sender != _owner
        if (_owner != address(this)) {
            vm.expectEmit(true, true, false, false);
            emit OwnershipTransferred(address(this), _owner);
            ++numOfEvents;
        }

        // then the event `ConsensusCreated` will be emitted
        vm.expectEmit(false, false, false, true);
        emit ConsensusCreated(_owner, _inputBox);
        ++numOfEvents;

        vm.recordLogs();
        authority = new Authority(_owner, _inputBox);
        Vm.Log[] memory entries = vm.getRecordedLogs();

        assertEq(entries.length, numOfEvents, "number of events");
        assertEq(authority.owner(), _owner, "authority owner");
    }

    function testRevertsOwnerAddressZero(IInputBox _inputBox) public {
        vm.expectRevert("Ownable: new owner is the zero address");
        new Authority(address(0), _inputBox);
    }

    function testMigrateHistory(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _newConsensus
    ) public isMockable(address(_history)) {
        vm.assume(_owner != address(0));
        vm.assume(_newConsensus != address(0));

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

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

        vm.expectCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.migrateToConsensus.selector,
                _newConsensus
            )
        );

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

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

        vm.assume(address(_history) != address(authority));
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(IHistory.submitClaim.selector, _claim),
            ""
        );

        // will fail as not called from owner
        vm.expectRevert("Ownable: caller is not the owner");
        authority.submitClaim(_claim);

        vm.expectCall(
            address(_history),
            abi.encodeWithSelector(IHistory.submitClaim.selector, _claim)
        );

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

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        vm.expectEmit(false, false, false, true);
        emit NewHistory(_history);
        authority.setHistory(_history);

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

    function testGetClaim(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _dapp,
        bytes calldata _proofContext,
        bytes32 _r0,
        uint256 _r1,
        uint256 _r2
    ) public isMockable(address(_history)) {
        vm.assume(_owner != address(0));
        vm.assume(_owner != address(this));

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

        // mocking history
        vm.assume(address(_history) != address(authority));
        vm.mockCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.getClaim.selector,
                _dapp,
                _proofContext
            ),
            abi.encode(_r0, _r1, _r2)
        );

        vm.expectCall(
            address(_history),
            abi.encodeWithSelector(
                IHistory.getClaim.selector,
                _dapp,
                _proofContext
            )
        );

        // perform call
        (bytes32 r0, uint256 r1, uint256 r2) = authority.getClaim(
            _dapp,
            _proofContext
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
        IHistory _newHistory,
        address _dapp,
        bytes calldata _claim,
        address _consensus,
        bytes calldata _proofContext
    ) public {
        vm.assume(_owner != address(0));

        HistoryReverts historyR = new HistoryReverts();

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(historyR);
        assertEq(address(authority.getHistory()), address(historyR));

        vm.expectRevert();
        vm.prank(_owner);
        authority.submitClaim(_claim);

        vm.expectRevert();
        vm.prank(_owner);
        authority.migrateHistoryToConsensus(_consensus);

        vm.expectRevert();
        authority.getClaim(_dapp, _proofContext);

        vm.prank(_owner);
        authority.setHistory(_newHistory);
        assertEq(address(authority.getHistory()), address(_newHistory));
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

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

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

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

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

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

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
        vm.expectRevert(AuthorityWithdrawalFailed.selector);
        authority.withdrawERC20Tokens(tokenFailed, _recipient, _amount);

        // after failed withdraw. All balances stay the same
        assertEq(tokenFailed.balanceOf(address(authority)), _balance);
        assertEq(tokenFailed.balanceOf(_recipient), 0);
    }

    function testJoin(
        address _owner,
        IInputBox _inputBox,
        IHistory _history,
        address _dapp
    ) public {
        vm.assume(_owner != address(0));

        authority = new Authority(_owner, _inputBox);

        vm.prank(_owner);
        authority.setHistory(_history);

        vm.expectEmit(false, false, false, true);
        emit ApplicationJoined(_dapp);

        vm.prank(_dapp);
        authority.join();
    }
}

contract AuthorityHandler is Test {
    struct Claim {
        bytes32 epochHash;
        uint128 firstIndex;
        uint128 lastIndex;
    }
    struct ClaimContext {
        address dapp;
        Claim claim;
        uint256 proofContext;
    }

    Authority immutable authority;
    History history; // current history
    History[] histories; // histories that have been used
    History[] backUpHistories; // new histories that are ready to be used

    mapping(History => ClaimContext[]) claimContext; // history => ClaimContext[]
    mapping(History => mapping(address => uint256)) numClaims; // history => dapp => #claims
    mapping(History => mapping(address => uint256)) nextIndices; // history => dapp => index of the next input to be processed

    uint128 constant MAXIDX = type(uint128).max;

    constructor(
        History _history,
        Authority _authority,
        History[] memory _backUpHistories
    ) {
        history = _history;
        histories.push(history);
        authority = _authority;
        backUpHistories = _backUpHistories;
    }

    function submitClaim(address _dapp, Claim memory _claim) external {
        uint256 firstIndex = nextIndices[history][_dapp];

        // We need to represent `firstIndex` in a uint128
        if (firstIndex > MAXIDX) return;

        // `lastIndex` needs to be greater than or equal to `firstIndex` and
        // also fit in a `uint128`
        uint256 lastIndex = bound(_claim.lastIndex, firstIndex, MAXIDX);

        _claim.firstIndex = uint128(firstIndex);
        _claim.lastIndex = uint128(lastIndex);

        bytes memory encodedData = abi.encode(_dapp, _claim);

        if (address(authority) != history.owner()) {
            vm.expectRevert("Ownable: caller is not the owner");
            authority.submitClaim(encodedData);
            return;
        }

        authority.submitClaim(encodedData);

        // Get the claim index and increment the number of claims
        uint256 claimIndex = numClaims[history][_dapp]++;

        claimContext[history].push(ClaimContext(_dapp, _claim, claimIndex));

        // Here we are not worried about overflowing 'lastIndex` because
        // it is a `uint256` guaranteed to fit in a `uint128`
        nextIndices[history][_dapp] = lastIndex + 1;
    }

    function migrateHistoryToConsensus(address _consensus) external {
        if (address(authority) != history.owner()) {
            vm.expectRevert("Ownable: caller is not the owner");
        } else if (_consensus == address(0)) {
            vm.expectRevert("Ownable: new owner is the zero address");
        }
        authority.migrateHistoryToConsensus(_consensus);
    }

    function setNewHistory() external {
        // take a back up new history from array
        if (backUpHistories.length > 0) {
            history = backUpHistories[backUpHistories.length - 1];
            backUpHistories.pop();
            authority.setHistory(history);
            histories.push(history);
        }
    }

    function setSameHistory() external {
        authority.setHistory(history);
    }

    function setOldHistory(uint256 _index) external {
        // pick a random old history
        // this should not raise a division-by-zero error because
        // the `histories` array is guaranteed to have at least one
        // history from construction
        history = histories[_index % histories.length];

        // with 50% chance randomly migrate the history to the authority
        // this will help cover the cases where authority is not the owner
        // of the history contract
        if (_index % 2 == 0) {
            vm.prank(history.owner());
            history.migrateToConsensus(address(authority));
        }

        authority.setHistory(history);
    }

    function checkHistory() external {
        assertEq(
            address(history),
            address(authority.getHistory()),
            "check history"
        );
    }

    function checkClaimAux(
        ClaimContext memory selectedClaimContext,
        bytes32 returnedEpochHash,
        uint256 returnedFirstIndex,
        uint256 returnedLastIndex
    ) internal {
        assertEq(
            returnedEpochHash,
            selectedClaimContext.claim.epochHash,
            "check epoch hash"
        );
        assertEq(
            returnedFirstIndex,
            selectedClaimContext.claim.firstIndex,
            "check first index"
        );
        assertEq(
            returnedLastIndex,
            selectedClaimContext.claim.lastIndex,
            "check last index"
        );
    }

    function checkClaim(
        uint256 _historyIndex,
        uint256 _claimContextIndex
    ) external {
        // this should not raise a division-by-zero error because
        // the `histories` array is guaranteed to have at least one
        // history from construction
        History selectedHistory = histories[_historyIndex % histories.length];

        // skip if history has no claim
        uint256 numClaimContexts = claimContext[selectedHistory].length;
        if (numClaimContexts == 0) return;

        ClaimContext memory selectedClaimContext = claimContext[
            selectedHistory
        ][_claimContextIndex % numClaimContexts];

        bytes32 returnedEpochHash;
        uint256 returnedFirstIndex;
        uint256 returnedLastIndex;

        (
            returnedEpochHash,
            returnedFirstIndex,
            returnedLastIndex
        ) = selectedHistory.getClaim(
            selectedClaimContext.dapp,
            abi.encode(selectedClaimContext.proofContext)
        );

        checkClaimAux(
            selectedClaimContext,
            returnedEpochHash,
            returnedFirstIndex,
            returnedLastIndex
        );

        if (address(selectedHistory) == address(authority.getHistory())) {
            // selected history is the current history
            // also check that call through authority returns the same claim
            (
                returnedEpochHash,
                returnedFirstIndex,
                returnedLastIndex
            ) = authority.getClaim(
                selectedClaimContext.dapp,
                abi.encode(selectedClaimContext.proofContext)
            );

            checkClaimAux(
                selectedClaimContext,
                returnedEpochHash,
                returnedFirstIndex,
                returnedLastIndex
            );
        }
    }

    // view functions
    function getNumHistories() external view returns (uint256) {
        return histories.length;
    }

    function getNumClaimContext(
        uint256 _historyIndex
    ) external view returns (uint256) {
        return claimContext[histories[_historyIndex]].length;
    }
}

contract AuthorityInvariantTest is Test {
    AuthorityHandler handler;
    History[] backUpHistories;

    function setUp() public {
        // this setup is only for invariant testing
        address inputBox = vm.addr(uint256(keccak256("inputBox")));
        Authority auth = new Authority(address(this), IInputBox(inputBox));
        History hist = new History(address(auth));
        auth.setHistory(hist);

        // back up new histories
        for (uint256 i; i < 30; ++i) {
            backUpHistories.push(new History(address(auth)));
        }

        handler = new AuthorityHandler(hist, auth, backUpHistories);
        auth.transferOwnership(address(handler));

        targetContract(address(handler));
    }

    function invariantTests() external {
        // check all claims
        for (uint256 i; i < handler.getNumHistories(); ++i) {
            for (uint256 j; j < handler.getNumClaimContext(i); ++j) {
                handler.checkClaim(i, j);
            }
        }
    }
}
