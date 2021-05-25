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
    mapping(address =>  uint) etherBalanceOf;

    //token balances of L2 addresses
    mapping(address => mapping(address => uint)) erc20BalanceOf;

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
    // the off-chain machine has a 8 byte word
    function addInput(bytes calldata _input, uint _op)
        public
        override
        noReentrancy()
        returns (bytes32)
    {
        require(_input.length > 0, "input length should be greater than 0");
        require((inputBlobBox.length + 1) <= 10, "input box size cannot be greater than 10");

        if(Operation(_op) == Operation.EtherOp){
            (
                Operation _operation,
                Transaction _transaction,
                address[] memory _receivers,
                uint256[] memory _amounts,
                bytes memory _data
            ) = abi.decode(_input, (Operation, Transaction, address [], uint256[], bytes));

            inputBlobBox.push(
                InputBlob(_operation, _transaction, _receivers, _amounts, address(0), msg.sender)
            );

            emit EtherInputAdded(_operation, _transaction, _receivers, _amounts);
        }

        if(Operation(_op) == Operation.ERC20Op){
            (
                Operation _operation,
                Transaction _transaction,
                address[] memory _receivers,
                uint256[] memory _amounts,
                address _ERC20,
                bytes memory _data
            ) = abi.decode(_input, (Operation, Transaction, address [], uint256 [], address, bytes));

            inputBlobBox.push(
                InputBlob(_operation, _transaction , _receivers, _amounts, _ERC20, msg.sender)
            );
        }


        if(inputBlobBox.length == 10){
            processBatchInputs();
        }
        // when input box is 10
        // process each input one after the other.
        // debit and credit based on the balance of each address
        // ensure that the sender is the portal when it's deposit
        // But for transfer and withdraws, we should check that the transaction has been sent by the holder.
        bytes memory metadata = abi.encode(msg.sender, block.timestamp);
        bytes32 inputHash = keccak256(abi.encode(keccak256(metadata), keccak256(_input)));
        return inputHash;
    }

    function processBatchInputs() internal returns (bool) {
        for(uint i = 0; i < inputBlobBox.length; i++){
            InputBlob memory inputBlob = inputBlobBox[i];

            if(inputBlob.operation == Operation.EtherOp){
                if(inputBlob.transaction == Transaction.Deposit){
                    for(uint j = 0;j < inputBlob.receivers.length; j++){
                        address receiver = inputBlob.receivers[j];
                        if(inputBlob.sender == portalContract){
                            uint amount = inputBlob.amounts[j];
                            inputBlob.amounts[j] = 0;
                            etherBalanceOf[receiver] += amount;
                        }
                    }
                }

                if(inputBlob.transaction == Transaction.Transfer){
                    for(uint  j = 0; j < inputBlob.receivers.length; j++){
                        address receiver = inputBlob.receivers[j];
                        if(msg.sender == inputBlob.sender){
                            uint amount = inputBlob.amounts[j];
                            uint senderBalance = etherBalanceOf[msg.sender] - amount; // use safeMath here??
                            if(senderBalance > 0){ // what happens when user balance reaches 0 and there's still an input to process?
                                inputBlob.amounts[j] = 0;
                                etherBalanceOf[msg.sender] -= amount;
                                etherBalanceOf[receiver] += amount;
                            }
                        }
                    }
                }

                if(inputBlob.transaction == Transaction.Withdraw){

                }
            }

            if(inputBlob.operation == Operation.ERC20Op){
                if(inputBlob.transaction == Transaction.Deposit){
                    for(uint j = 0;j < inputBlob.receivers.length; j++){
                        address recipient = inputBlob.receivers[j];
                        if(inputBlob.sender == portalContract){
                            uint amount = inputBlob.amounts[j];
                            inputBlob.amounts[j] = 0;
                            erc20BalanceOf[recipient][inputBlob._ERC20] += amount;
                        }
                    }
                }

                if(inputBlob.transaction == Transaction.Transfer){
                    for(uint  j = 0; j < inputBlob.receivers.length; j++){
                        address receiver = inputBlob.receivers[j];
                        if(msg.sender == inputBlob.sender){
                            uint amount = inputBlob.amounts[j];
                            uint senderBalance =
                                erc20BalanceOf[msg.sender][inputBlob._ERC20] - amount;
                            if(senderBalance > 0){
                                inputBlob.amounts[j] = 0;
                                erc20BalanceOf[msg.sender][inputBlob._ERC20] -= amount;
                                erc20BalanceOf[receiver][inputBlob._ERC20] += amount;
                            }
                        }
                    }
                }

                if(inputBlob.transaction == Transaction.Withdraw){

                }
            }
        }

        return true;
    }

    /// @notice get input inside inbox of currently proposed claim
    /// @param _index index of input inside that inbox
    /// @return hash of input at index _index
    /// @dev currentInputBox being zero means that the inputs for
    ///      the claimed epoch are on input box one
    function getInput(uint256 _index) public view override returns (bytes memory) {
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
    /// @dev can only be called by DescartesV2 contract
    function onNewEpoch() public override {
        // clear input box for new inputs
        // the current input box should be accumulating inputs
        // for the new epoch already. So we clear the other one.
        delete inputBlobBox;
    }
}
