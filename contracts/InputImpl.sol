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
pragma solidity ^0.7.0;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// https://github.com/GNSPS/solidity-bytes-utils
import "solidity-bytes-utils/contracts/BytesLib.sol";

import "@cartesi/logger/contracts/LoggerInterface.sol";

import "./Input.sol";
import "./DescartesV2.sol";

// TODO: this contract seems to be very unsafe, need to think about security implications
contract InputImpl is Input {
    using SafeMath for uint256;
    using BytesLib for bytes;

    DescartesV2 immutable descartesV2; // descartes 2 contract using this input contract
    LoggerInterface immutable logger; // logger contract
    uint64 immutable log2size; // log2size of input flashdrive

    // always needs to keep track of two input boxes:
    // 1 for the input accumulation of next epoch
    // and 1 for the messages during current epoch. To save gas we alternate
    // between inputBox0 and inputBox1
    bytes32[] inputBox0;
    bytes32[] inputBox1;

    uint256 currentInputBox;
    // @notice functions modified by onlyDescartesV2 will only be executed if
    // they're called by DescartesV2 contract, otherwise it will throw an exception
    modifier onlyDescartesV2 {
        require(
            msg.sender == address(descartesV2),
            "Only descartesV2 can call this functions"
        );
        _;
    }

    constructor(address _descartesV2, address _logger) {
        descartesV2 = DescartesV2(_descartesV2);
        logger = LoggerInterface(_logger);
    }

    /// @dev offchain code is responsible for making sure
    ///      that input size plus msg.sender and block timestamp
    ///      is power of 2 and multiple of 8 since the offchain machine
    ///      has a 8 byte word
    function addInput(bytes calldata _input) public override returns (bytes32) {
        require(_input.length > 0, "input is empty");

        // lock to guard reentrancy
        bool lock;
        require(!lock, "Reentrancy not allowed");
        lock = true;

        // prepend msg.sender and block timestamp to _input
        bytes memory data = abi.encode(msg.sender, block.timestamp, _input);

        bytes8[] memory data64 = new bytes8[](data.length / 8);
        // transform bytes into bytes8[] array
        for (uint256 i = 0; i < data.length; i += 8) {
            data64[i / 8] = bytes8(data.toUint64(i));
        }

        // get merkle root hash of input
        bytes32 root = logger.calculateMerkleRootFromData(log2Size, data64);

        // notifyInput returns true if that input belongs
        // belong to a new epoch
        if (descartesV2.notifyInput()) {
            swapInputBox();
        }

        // add input to correct inbox
        currentInputBox == 0 ? inputBox0.push(root) : inputBox1.push(root);

        lock = false;
        // notify descartesV2 of new input
        return root;
    }

    // this has to check if state is input accumulation
    // otherwise it could be looking at the wrong inbox
    function getInput(uint256 _index) public override returns (bytes32) {
        return currentInputBox == 0 ? inputBox1[_index] : inputBox0[_index];
    }

    // new input accumulation has to be called even when there are no new
    // input but the epoch is over
    function onNewInputAccumulation() public override onlyDescartesV2 {
        swapInputBox();
    }

    function onNewEpoch() public override onlyDescartesV2 {
        // clear input box for new inputs
        // the current input box should be accumulating inputs
        // for the new epoch already. So we clear the other one.
        currentInputBox == 0 ? delete inputBox1 : delete inputBox0;
    }

    function swapInputBox() internal {
        currentInputBox == 0 ? currentInputBox = 1 : currentInputBox = 0;
    }
}
