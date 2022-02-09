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

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import "@cartesi/util/contracts/Merkle.sol";

import "./MockInput.sol";

// TODO: this contract seems to be very unsafe, need to think about security implications
contract MockInputImpl is MockInput {
    address immutable portalContract;

    bool lock; //reentrancy lock

    InputBlob[] inputBlobBox;

    struct InputBlob {
        Operation operation;
        Transaction transaction;
        address[] receivers;
        uint256[] amounts;
        address _ERC20;
        address sender;
    }

    //ether balance of L2 addresses
    mapping(address => uint256) etherBalanceOf;

    //token balances of L2 addresses
    mapping(address => mapping(address => uint256)) erc20BalanceOf;

    /// @notice functions modified by noReentrancy are not subject to recursion
    /// TODO: up for discussion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    constructor(address _portalContract) {
        portalContract = _portalContract;
    }

    /// @notice add input to processed by next epoch
    ///         it essentially mimics the epoch behavior
    /// @param _input input to be understood by off-chain machine
    /// @dev off-chain code is responsible for making sure
    ///      that input size is power of 2 and multiple of 8 since
    ///      the off-chain machine has a 8 byte word
    function addInput(bytes calldata _input, uint256 _op)
        public
        override
        noReentrancy()
        returns (bytes32)
    {
        require(
            _input.length > 0 && _input.length <= 512,
            "input length should be between 0 and 512"
        );
        require(
            (inputBlobBox.length + 1) <= 10,
            "input box size cannot be greater than 10"
        );

        if (Operation(_op) == Operation.EtherOp) {
            (
                Operation _operation,
                Transaction _transaction,
                address[] memory _receivers,
                uint256[] memory _amounts,
                bytes memory _data
            ) =
                abi.decode(
                    _input,
                    (Operation, Transaction, address[], uint256[], bytes)
                );

            inputBlobBox.push(
                InputBlob(
                    _operation,
                    _transaction,
                    _receivers,
                    _amounts,
                    address(0),
                    msg.sender
                )
            );

            emit EtherInputAdded(
                _operation,
                _transaction,
                _receivers,
                _amounts
            );
        }

        if (Operation(_op) == Operation.ERC20Op) {
            (
                Operation _operation,
                Transaction _transaction,
                address[] memory _receivers,
                uint256[] memory _amounts,
                address _ERC20,
                bytes memory _data
            ) =
                abi.decode(
                    _input,
                    (
                        Operation,
                        Transaction,
                        address[],
                        uint256[],
                        address,
                        bytes
                    )
                );

            inputBlobBox.push(
                InputBlob(
                    _operation,
                    _transaction,
                    _receivers,
                    _amounts,
                    _ERC20,
                    msg.sender
                )
            );

            emit Erc20InputAdded(
                _operation,
                _transaction,
                _receivers,
                _amounts,
                _ERC20
            );
        }

        if (inputBlobBox.length == 10) {
            processBatchInputs();
        }
        // when input box is 10
        // process each input one after the other.
        // debit and credit based on the balance of each address
        // ensure that the sender is the portal when it's deposit
        // But for transfer and withdraws, we should check that the transaction has been sent by the holder.
        bytes memory metadata = abi.encode(msg.sender, block.timestamp);
        bytes32 inputHash =
            keccak256(abi.encode(keccak256(metadata), keccak256(_input)));
        return inputHash;
    }

    function processBatchInputs() internal returns (bool) {
        for (uint256 i = 0; i < inputBlobBox.length; i++) {
            InputBlob memory inputBlob = inputBlobBox[i];

            if (inputBlob.operation == Operation.EtherOp) {
                if (inputBlob.transaction == Transaction.Deposit) {
                    for (uint256 j = 0; j < inputBlob.receivers.length; j++) {
                        address receiver = inputBlob.receivers[j];
                        if (inputBlob.sender == portalContract) {
                            uint256 amount = inputBlob.amounts[j];
                            etherBalanceOf[receiver] += amount;
                        }
                    }
                }

                if (inputBlob.transaction == Transaction.Transfer) {
                    for (uint256 j = 0; j < inputBlob.receivers.length; j++) {
                        address receiver = inputBlob.receivers[j];
                        uint256 amount = inputBlob.amounts[j];
                        if (etherBalanceOf[inputBlob.sender] >= amount) {
                            etherBalanceOf[inputBlob.sender] -= amount;
                            etherBalanceOf[receiver] += amount;
                        }
                    }
                }
            }

            if (inputBlob.operation == Operation.ERC20Op) {
                if (inputBlob.transaction == Transaction.Deposit) {
                    for (uint256 j = 0; j < inputBlob.receivers.length; j++) {
                        address recipient = inputBlob.receivers[j];
                        if (inputBlob.sender == portalContract) {
                            uint256 amount = inputBlob.amounts[j];
                            erc20BalanceOf[recipient][
                                inputBlob._ERC20
                            ] += amount;
                        }
                    }
                }

                if (inputBlob.transaction == Transaction.Transfer) {
                    for (uint256 j = 0; j < inputBlob.receivers.length; j++) {
                        address receiver = inputBlob.receivers[j];
                        uint256 amount = inputBlob.amounts[j];

                        if (
                            erc20BalanceOf[inputBlob.sender][
                                inputBlob._ERC20
                            ] >= amount
                        ) {
                            erc20BalanceOf[inputBlob.sender][
                                inputBlob._ERC20
                            ] -= amount;
                            erc20BalanceOf[receiver][
                                inputBlob._ERC20
                            ] += amount;
                        }
                    }
                }
            }
        }

        //finalize the epoch after processing 10 inputs with the Output contract on epoch

        return true;
    }

    /// @notice get input inside inbox of currently proposed claim
    /// @param _index index of input inside that inbox
    /// @return hash of input at index _index
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getInput(uint256 _index)
        public
        view
        override
        returns (bytes memory)
    {
        InputBlob memory inputBlob = inputBlobBox[_index];
        return abi.encode(inputBlob);
    }

    /// @notice get number of inputs inside inbox of currently proposed claim
    /// @return number of inputs on that input box
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getNumberOfInputs() public view override returns (uint256) {
        return inputBlobBox.length;
    }

    /// @notice called when a new epoch begins, clears deprecated inputs
    /// @dev can only be called by Rollups contract
    function onNewEpoch() public override {
        // clear input box for new inputs
        // the current input box should be accumulating inputs
        // for the new epoch already. So we clear the other one.
        delete inputBlobBox;
    }
}
