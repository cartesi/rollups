// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Implementation
pragma solidity ^0.8.0;

import "./Input.sol";
import "./Rollups.sol";

contract InputImpl is Input {
    Rollups immutable rollups; // rollups contract using this input contract

    // always needs to keep track of two input boxes:
    // 1 for the input accumulation of next epoch
    // and 1 for the messages during current epoch. To save gas we alternate
    // between inputBox0 and inputBox1
    bytes32[] inputBox0;
    bytes32[] inputBox1;

    uint256 immutable inputDriveSize; // size of input flashdrive
    uint256 currentInputBox;

    /// @param _rollups address of rollups contract that will manage inboxes
    /// @param _log2Size size of the input drive of the machine
    constructor(address _rollups, uint256 _log2Size) {
        require(_log2Size >= 3 && _log2Size <= 64, "log size: [3,64]");

        rollups = Rollups(_rollups);
        inputDriveSize = (1 << _log2Size);
    }

    /// @notice add input to processed by next epoch
    /// @param _input input to be understood by offchain machine
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    // the offchain machine has a 8 byte word
    function addInput(bytes calldata _input) public override returns (bytes32) {
        require(
            _input.length > 0 && _input.length <= inputDriveSize,
            "input len: (0,driveSize]"
        );

        // notifyInput returns true if that input
        // belongs to a new epoch
        if (rollups.notifyInput()) {
            swapInputBox();
        }

        // points to correct inputBox
        bytes32[] storage inputBox = currentInputBox == 0 ? inputBox0 : inputBox1;

        // keccak 64 bytes into 32 bytes
        bytes32 keccakMetadata =
            keccak256(
                abi.encode(
                    msg.sender,
                    block.number,
                    block.timestamp,
                    rollups.getCurrentEpoch(), // epoch index
                    inputBox.length // input index
                )
            );

        bytes32 keccakInput = keccak256(_input);

        bytes32 inputHash = keccak256(abi.encode(keccakMetadata, keccakInput));

        // add input to correct inbox
        inputBox.push(inputHash);

        emit InputAdded(
            rollups.getCurrentEpoch(),
            inputBox.length - 1,
            msg.sender,
            block.timestamp,
            _input
        );

        return inputHash;
    }

    /// @notice get input inside inbox of currently proposed claim
    /// @param _index index of input inside that inbox
    /// @return hash of input at index _index
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getInput(uint256 _index) public view override returns (bytes32) {
        return currentInputBox == 0 ? inputBox1[_index] : inputBox0[_index];
    }

    /// @notice get number of inputs inside inbox of currently proposed claim
    /// @return number of inputs on that input box
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getNumberOfInputs() public view override returns (uint256) {
        return currentInputBox == 0 ? inputBox1.length : inputBox0.length;
    }

    /// @notice get inbox currently receiveing inputs
    /// @return input inbox currently receiveing inputs
    function getCurrentInbox() public view override returns (uint256) {
        return currentInputBox;
    }

    /// @notice called when a new input accumulation phase begins
    ///         swap inbox to receive inputs for upcoming epoch
    /// @dev can only be called by Rollups contract
    function onNewInputAccumulation() public override {
        onlyRollups();
        swapInputBox();
    }

    /// @notice called when a new epoch begins, clears deprecated inputs
    /// @dev can only be called by Rollups contract
    function onNewEpoch() public override {
        // clear input box for new inputs
        // the current input box should be accumulating inputs
        // for the new epoch already. So we clear the other one.
        onlyRollups();
        currentInputBox == 0 ? delete inputBox1 : delete inputBox0;
    }

    /// @notice check if message sender is Rollups
    function onlyRollups() internal view {
        require(msg.sender == address(rollups), "Only rollups");
    }

    /// @notice changes current input box
    function swapInputBox() internal {
        currentInputBox == 0 ? currentInputBox = 1 : currentInputBox = 0;
    }
}
