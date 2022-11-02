// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input library
pragma solidity ^0.8.0;

import {LibRollups} from "../libraries/LibRollups.sol";

library LibInput {
    using LibRollups for LibRollups.DiamondStorage;

    bytes32 constant DIAMOND_STORAGE_POSITION =
        keccak256("Input.diamond.storage");

    struct DiamondStorage {
        // always needs to keep track of two input boxes:
        // 1 for the input accumulation of next epoch
        // and 1 for the messages during current epoch. To save gas we alternate
        // between inputBox0 and inputBox1
        bytes32[] inputBox0;
        bytes32[] inputBox1;
        uint256 inputDriveSize; // size of input flashdrive
        uint256 currentInputBox;
    }

    function diamondStorage()
        internal
        pure
        returns (DiamondStorage storage ds)
    {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    /// @notice get input inside inbox of currently proposed claim
    /// @param ds diamond storage pointer
    /// @param index index of input inside that inbox
    /// @return hash of input at index index
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getInput(
        DiamondStorage storage ds,
        uint256 index
    ) internal view returns (bytes32) {
        return
            ds.currentInputBox == 0 ? ds.inputBox1[index] : ds.inputBox0[index];
    }

    /// @notice get number of inputs inside inbox of currently proposed claim
    /// @param ds diamond storage pointer
    /// @return number of inputs on that input box
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getNumberOfInputs(
        DiamondStorage storage ds
    ) internal view returns (uint256) {
        return
            ds.currentInputBox == 0 ? ds.inputBox1.length : ds.inputBox0.length;
    }

    /// @notice add input to processed by next epoch
    /// @param ds diamond storage pointer
    /// @param input input to be understood by offchain machine
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    ///      the offchain machine has a 8 byte word
    function addInput(
        DiamondStorage storage ds,
        bytes memory input
    ) internal returns (bytes32) {
        return addInputFromSender(ds, input, msg.sender);
    }

    /// @notice add internal input to processed by next epoch
    /// @notice this function is to be reserved for internal usage only
    /// @notice for normal inputs, call `addInput` instead
    /// @param ds diamond storage pointer
    /// @param input input to be understood by offchain machine
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    ///      the offchain machine has a 8 byte word
    function addInternalInput(
        DiamondStorage storage ds,
        bytes memory input
    ) internal returns (bytes32) {
        return addInputFromSender(ds, input, address(this));
    }

    /// @notice add input from a specific sender to processed by next epoch
    /// @notice this function is to be reserved for internal usage only
    /// @notice for normal inputs, call `addInput` instead
    /// @param ds diamond storage pointer
    /// @param input input to be understood by offchain machine
    /// @param sender input sender address
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    ///      the offchain machine has a 8 byte word
    function addInputFromSender(
        DiamondStorage storage ds,
        bytes memory input,
        address sender
    ) internal returns (bytes32) {
        LibRollups.DiamondStorage storage rollupsDS = LibRollups
            .diamondStorage();

        require(input.length <= ds.inputDriveSize, "input len: [0,driveSize]");

        // notifyInput returns true if that input
        // belongs to a new epoch
        if (rollupsDS.notifyInput()) {
            swapInputBox(ds);
        }

        // points to correct inputBox
        bytes32[] storage inputBox = ds.currentInputBox == 0
            ? ds.inputBox0
            : ds.inputBox1;

        // get current epoch index
        uint256 currentEpoch = rollupsDS.getCurrentEpoch();

        // keccak 64 bytes into 32 bytes
        bytes32 keccakMetadata = keccak256(
            abi.encode(
                sender,
                block.number,
                block.timestamp,
                currentEpoch, // epoch index
                inputBox.length // input index
            )
        );

        bytes32 keccakInput = keccak256(input);

        bytes32 inputHash = keccak256(abi.encode(keccakMetadata, keccakInput));

        // add input to correct inbox
        inputBox.push(inputHash);

        emit InputAdded(
            currentEpoch,
            inputBox.length - 1,
            sender,
            block.timestamp,
            input
        );

        return inputHash;
    }

    /// @notice called when a new input accumulation phase begins
    ///         swap inbox to receive inputs for upcoming epoch
    /// @param ds diamond storage pointer
    function onNewInputAccumulation(DiamondStorage storage ds) internal {
        swapInputBox(ds);
    }

    /// @notice called when a new epoch begins, clears deprecated inputs
    /// @param ds diamond storage pointer
    function onNewEpoch(DiamondStorage storage ds) internal {
        // clear input box for new inputs
        // the current input box should be accumulating inputs
        // for the new epoch already. So we clear the other one.
        ds.currentInputBox == 0 ? delete ds.inputBox1 : delete ds.inputBox0;
    }

    /// @notice changes current input box
    /// @param ds diamond storage pointer
    function swapInputBox(DiamondStorage storage ds) internal {
        ds.currentInputBox = (ds.currentInputBox == 0) ? 1 : 0;
    }

    /// @notice input added
    /// @param epochNumber which epoch this input belongs to
    /// @param inputIndex index of the input just added
    /// @param sender msg.sender
    /// @param timestamp block.timestamp
    /// @param input input data
    event InputAdded(
        uint256 indexed epochNumber,
        uint256 indexed inputIndex,
        address sender,
        uint256 timestamp,
        bytes input
    );
}
