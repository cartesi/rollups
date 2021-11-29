// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Rollups initialization facet
pragma solidity ^0.8.0;

import {IInput} from "../interfaces/IInput.sol";

import {LibInput} from "../libraries/LibInput.sol";
import {LibRollups} from "../libraries/LibRollups.sol";

contract InputFacet is IInput {
    /// @notice add input to processed by next epoch
    /// @param _input input to be understood by offchain machine
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    // the offchain machine has a 8 byte word
    function addInput(bytes calldata _input) public override returns (bytes32) {
        LibInput.DiamondStorage storage ds = LibInput.diamondStorage();

        require(
            _input.length > 0 && _input.length <= ds.inputDriveSize,
            "input len: (0,driveSize]"
        );

        // keccak 64 bytes into 32 bytes
        bytes32 keccakMetadata =
            keccak256(abi.encode(msg.sender, block.timestamp));
        bytes32 keccakInput = keccak256(_input);

        bytes32 inputHash = keccak256(abi.encode(keccakMetadata, keccakInput));

        // notifyInput returns true if that input
        // belongs to a new epoch
        if (LibRollups.notifyInput()) {
            LibInput.swapInputBox();
        }

        // add input to correct inbox
        ds.currentInputBox == 0
            ? ds.inputBox0.push(inputHash)
            : ds.inputBox1.push(inputHash);

        emit InputAdded(
            LibRollups.getCurrentEpoch(),
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
        LibInput.DiamondStorage storage ds = LibInput.diamondStorage();
        return
            ds.currentInputBox == 0
                ? ds.inputBox1[_index]
                : ds.inputBox0[_index];
    }

    /// @notice get number of inputs inside inbox of currently proposed claim
    /// @return number of inputs on that input box
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getNumberOfInputs() public view override returns (uint256) {
        LibInput.DiamondStorage storage ds = LibInput.diamondStorage();
        return
            ds.currentInputBox == 0 ? ds.inputBox1.length : ds.inputBox0.length;
    }

    /// @notice get inbox currently receiveing inputs
    /// @return input inbox currently receiveing inputs
    function getCurrentInbox() public view override returns (uint256) {
        LibInput.DiamondStorage storage ds = LibInput.diamondStorage();
        return ds.currentInputBox;
    }
}
