// Copyright (C) 2020 Cartesi Pte. Ltd.

// SPDX-License-Identifier: GPL-3.0-only
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

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

    // @notice emitted on adding Ether input
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
