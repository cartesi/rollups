// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Box Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {CanonicalMachine} from "contracts/common/CanonicalMachine.sol";
import {LibInput} from "contracts/library/LibInput.sol";

contract InputBoxTest is Test {
    using CanonicalMachine for CanonicalMachine.Log2Size;

    InputBox inputBox;

    event InputAdded(address indexed dapp, address sender, bytes input);

    function setUp() public {
        inputBox = new InputBox();
    }

    function testNoInputs(address _dapp) public {
        assertEq(inputBox.getNumberOfInputs(_dapp), 0);
    }

    // fuzz testing with multiple inputs
    function testAddInput(address _dapp, bytes[] calldata _inputs) public {
        uint256 numInputs = _inputs.length;
        bytes32[] memory returnedValues = new bytes32[](numInputs);
        uint256 year2022 = 1641070800; // Unix Timestamp for 2022

        // assume #bytes for each input is within bounds
        for (uint256 i; i < numInputs; ++i) {
            vm.assume(
                _inputs[i].length <=
                    (1 << CanonicalMachine.INPUT_MAX_LOG2_SIZE.uint64OfSize())
            );
        }

        // adding inputs
        for (uint256 i; i < numInputs; ++i) {
            // test for different block number and timestamp
            vm.roll(i);
            vm.warp(i + year2022); // year 2022

            // topic 1 is indexed; topic 2 and 3 aren't; check event data
            vm.expectEmit(true, false, false, true, address(inputBox));

            // The event we expect
            emit InputAdded(_dapp, address(this), _inputs[i]);

            returnedValues[i] = inputBox.addInput(_dapp, _inputs[i]);

            // test whether the number of inputs has increased
            assertEq(i + 1, inputBox.getNumberOfInputs(_dapp));
        }

        // testing added inputs
        for (uint256 i; i < numInputs; ++i) {
            // compute input hash for each input
            bytes32 inputHash = LibInput.computeInputHash(
                address(this),
                i, // block.number
                i + year2022, // block.timestamp
                _inputs[i],
                i // inputBox.length
            );
            // test if input hash is the same as in InputBox
            assertEq(inputHash, inputBox.getInputHash(_dapp, i));
            // test if input hash is the same as returned from calling addInput() function
            assertEq(inputHash, returnedValues[i]);
        }
    }
}
