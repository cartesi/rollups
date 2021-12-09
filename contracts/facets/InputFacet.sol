// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input facet
pragma solidity ^0.8.0;

import {IInput} from "../interfaces/IInput.sol";

import {LibInput} from "../libraries/LibInput.sol";

contract InputFacet is IInput {
    using LibInput for LibInput.DiamondStorage;

    /// @notice add input to processed by next epoch
    /// @param _input input to be understood by offchain machine
    /// @dev offchain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    //       the offchain machine has a 8 byte word
    function addInput(bytes calldata _input) public override returns (bytes32) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        return inputDS.addInput(_input);
    }

    /// @notice get input inside inbox of currently proposed claim
    /// @param _index index of input inside that inbox
    /// @return hash of input at index _index
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getInput(uint256 _index) public view override returns (bytes32) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        return inputDS.getInput(_index);
    }

    /// @notice get number of inputs inside inbox of currently proposed claim
    /// @return number of inputs on that input box
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getNumberOfInputs() public view override returns (uint256) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        return inputDS.getNumberOfInputs();
    }

    /// @notice get inbox currently receiveing inputs
    /// @return input inbox currently receiveing inputs
    function getCurrentInbox() public view override returns (uint256) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        return inputDS.currentInputBox;
    }
}
