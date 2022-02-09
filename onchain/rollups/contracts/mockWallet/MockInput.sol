// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Input
pragma solidity >=0.7.0;

interface MockInput {
    // Ether - deposits/withdrawal of ether
    // ERC20 - deposit/withdrawal of ERC20 compatible tokens
    enum Operation {EtherOp, ERC20Op}

    // Deposit - deposit from an L1 address to an L2 address
    // Transfer - transfer from one L2 address to another
    // Withdraw - withdraw from an L2 address to an L1 address
    enum Transaction {Deposit, Transfer, Withdraw}

    /// @notice emitted on adding Ether input
    event EtherInputAdded(
        Operation _operation,
        Transaction _transaction,
        address[] _receivers,
        uint256[] _amounts
    );

    event Erc20InputAdded(
        Operation _operation,
        Transaction _transaction,
        address[] _receivers,
        uint256[] _amounts,
        address _ERC20
    );

    /// @notice adds input to correct inbox
    /// @param _input bytes array of input
    /// @return merkle root hash of input
    /// @dev  msg.sender and timestamp are preppended log2 size
    ///       has to be calculated offchain taking that into account
    function addInput(bytes calldata _input, uint256 _operation)
        external
        returns (bytes32);

    /// @notice returns input from correct input inbox
    /// @param _index position of the input on inbox
    /// @return root hash of input
    function getInput(uint256 _index) external view returns (bytes memory);

    /// @notice returns number of inputs on correct inbox
    /// @return number of inputs of non active inbox
    function getNumberOfInputs() external view returns (uint256);

    /// @notice called when a new epoch begins, clears correct input box
    function onNewEpoch() external;
}
