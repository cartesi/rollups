// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input Box Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {CanonicalMachine} from "contracts/common/CanonicalMachine.sol";
import {LibInput} from "contracts/library/LibInput.sol";

contract InputBoxHandler is Test {
    IInputBox immutable inputBox;

    struct InputData {
        address dapp;
        uint256 index;
        bytes32 inputHash;
    }

    InputData[] inputDataArray;

    // array of addresses of dapps whose inboxes aren't empty
    address[] dapps;

    // mapping of dapp addresses to number of inputs
    mapping(address => uint256) numOfInputs;

    // block variables
    uint256 blockTimestamp = block.timestamp;
    uint256 blockNumber = block.number;

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function incrementBlockTimestamp() external {
        blockTimestamp++;
    }

    function incrementBlockNumber() external {
        blockNumber++;
    }

    function setBlockProperties() internal {
        vm.warp(blockTimestamp);
        vm.roll(blockNumber);
    }

    function addInput(address _dapp, bytes calldata _input) external {
        // For some reason, the invariant testing framework doesn't
        // record changes made to block properties, so we have to
        // set them in the beginning of every call
        setBlockProperties();

        // Get the index of the to-be-added input
        uint256 index = inputBox.getNumberOfInputs(_dapp);

        // Check if `getNumberOfInputs` matches internal count
        assertEq(index, numOfInputs[_dapp], "inbox size");

        // Make the sender add the input to the DApp's inbox
        vm.prank(msg.sender);
        bytes32 inputHash = inputBox.addInput(_dapp, _input);

        // If this is the first input being added to the DApp's inbox,
        // then push the dapp to the array of dapps
        if (index == 0) {
            dapps.push(_dapp);
        }

        // Increment the dapp's input count
        ++numOfInputs[_dapp];

        // Create the input data struct
        InputData memory inputData = InputData({
            dapp: _dapp,
            index: index,
            inputHash: inputHash
        });

        // Add the input data to the array
        inputDataArray.push(inputData);

        // Check if the inbox size increases by one
        assertEq(
            index + 1,
            inputBox.getNumberOfInputs(_dapp),
            "inbox size increment"
        );

        // Check if the input hash matches the one returned by `getInputHash`
        assertEq(
            inputHash,
            inputBox.getInputHash(_dapp, index),
            "returned input hash"
        );

        // Compute the input hash from the arguments passed to `addInput`
        bytes32 computedInputHash = LibInput.computeInputHash(
            msg.sender,
            block.number,
            block.timestamp,
            _input,
            index
        );

        // Check if the input hash matches the computed one
        assertEq(inputHash, computedInputHash, "computed input hash");
    }

    function getNumberOfInputs() external view returns (uint256) {
        return inputDataArray.length;
    }

    function getInputAt(uint256 _i) external view returns (InputData memory) {
        return inputDataArray[_i];
    }

    function getNumberOfDApps() external view returns (uint256) {
        return dapps.length;
    }

    function getDAppAt(uint256 _i) external view returns (address) {
        return dapps[_i];
    }

    function getNumberOfInputsForDApp(
        address _dapp
    ) external view returns (uint256) {
        return numOfInputs[_dapp];
    }
}

contract InputBoxTest is Test {
    using CanonicalMachine for CanonicalMachine.Log2Size;

    InputBox inputBox;
    InputBoxHandler handler;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inboxInputIndex,
        address sender,
        bytes input
    );

    function setUp() public {
        inputBox = new InputBox();
        handler = new InputBoxHandler(inputBox);

        // for the invariant testing,
        // don't call the input box contract directly
        // (do it through the handler contract)
        excludeContract(address(inputBox));
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

            // topics 1 and 2 are indexed; topic 3 isn't; check event data
            vm.expectEmit(true, true, false, true, address(inputBox));

            // The event we expect
            emit InputAdded(_dapp, i, address(this), _inputs[i]);

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

    function invariantInputData() external {
        // Get the total number of inputs
        uint256 totalNumOfInputs = handler.getNumberOfInputs();

        for (uint256 i; i < totalNumOfInputs; ++i) {
            // Get input data and metadata passed to `addInput`
            InputBoxHandler.InputData memory inputData = handler.getInputAt(i);

            // Make sure the input index is less than the inbox size
            assertLt(
                inputData.index,
                inputBox.getNumberOfInputs(inputData.dapp),
                "index bound check"
            );

            // Get the input hash returned by `getInputHash`
            bytes32 inputHash = inputBox.getInputHash(
                inputData.dapp,
                inputData.index
            );

            // Check if the input hash matches the one returned by `addInput`
            assertEq(inputHash, inputData.inputHash, "returned input hash");
        }

        // Get the number of dapps in the array
        uint256 numOfDApps = handler.getNumberOfDApps();

        // Check the input box size of all the dapps that
        // were interacted with, and sum them all up
        uint256 sum;
        for (uint256 i; i < numOfDApps; ++i) {
            address dapp = handler.getDAppAt(i);
            uint256 expected = handler.getNumberOfInputsForDApp(dapp);
            uint256 actual = inputBox.getNumberOfInputs(dapp);
            assertEq(expected, actual, "number of inputs for dapp");
            sum += actual;
        }
        assertEq(sum, totalNumOfInputs, "total number of inputs");
    }
}
