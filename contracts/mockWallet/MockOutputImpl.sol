// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Output Implementation
pragma solidity ^0.8.0;

import "@cartesi/util/contracts/Bitmask.sol";
import "@cartesi/util/contracts/Merkle.sol";

import "./MockOutput.sol";

contract MockOutputImpl is MockOutput {
    using Bitmask for mapping(uint256 => uint256);

    uint8 constant KECCAK_LOG2_SIZE = 5; // keccak log2 size

    address immutable rollups; // rollups contract using this validator

    mapping(uint256 => uint256) internal voucherBitmask;
    bytes32[] epochHashes;

    bool lock; //reentrancy lock

    /// @notice functions modified by noReentrancy are not subject to recursion
    modifier noReentrancy() {
        require(!lock, "reentrancy not allowed");
        lock = true;
        _;
        lock = false;
    }

    // @notice functions modified by onlyRollups will only be executed if
    // they're called by Rollups contract, otherwise it will throw an exception
    modifier onlyRollups {
        require(msg.sender == rollups, "Only rollups can call this functions");
        _;
    }

    // @notice creates OutputImpl contract
    // @params _rollups address of rollupscontract
    constructor(address _rollups) {
        rollups = _rollups;
    }

    /// @notice executes voucher
    /// @param _destination address that will execute voucher
    /// @param _payload payload to be executed by destination
    /// @param _epochIndex which epoch the voucher belongs to
    /// @param _inputIndex which input, inside the epoch, the voucher belongs to
    /// @param _voucherIndex index of voucher inside the input
    /// @param _voucherDriveHash hash of the vouchers drive where this voucher is contained
    /// @param _voucherProof bytes that describe the voucher, can encode different things
    /// @param _epochProof siblings of vouchers hash, to prove it is contained on epoch hash
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        uint256 _epochIndex,
        uint256 _inputIndex,
        uint256 _voucherIndex,
        bytes32 _voucherDriveHash,
        bytes32[] calldata _voucherProof,
        bytes32[] calldata _epochProof
    ) public override noReentrancy returns (bool) {
        uint256 bitmaskPosition =
            getBitMaskPosition(_voucherIndex, _inputIndex, _epochIndex);

        require(
            !voucherBitmask.getBit(bitmaskPosition),
            "voucher has already been executed"
        );

        bytes32 hashOfVoucher =
            keccak256(abi.encodePacked(_destination, _payload));

        // prove that the epoch contains that voucherdrive
        require(
            Merkle.getRootAfterReplacementInDrive(
                uint64(_voucherIndex * KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                64,
                hashOfVoucher,
                _voucherProof
            ) == _voucherDriveHash,
            "specific voucher is not contained in voucher drive merkle hash"
        );

        // prove that epoch hash contains the claimed vouchers hash
        require(
            Merkle.getRootAfterReplacementInDrive(
                uint64(_inputIndex * KECCAK_LOG2_SIZE),
                KECCAK_LOG2_SIZE,
                64,
                _voucherDriveHash,
                _epochProof
            ) == epochHashes[_epochIndex],
            "voucher drive hash not contained in epochHashes"
        );

        // do we need return data? emit event?
        (bool succ, bytes memory returnData) =
            address(_destination).call(_payload);

        if (succ) voucherBitmask.setBit(bitmaskPosition, true);

        return succ;
    }

    /// @notice called by rollups when an epoch is finalized
    /// @param _epochHash hash of finalized epoch
    /// @dev an epoch being finalized means that its vouchers can be called
    function onNewEpoch(bytes32 _epochHash) public override onlyRollups {
        epochHashes.push(_epochHash);
    }

    /// @notice get voucher position on bitmask
    /// @param _voucher of voucher inside the input
    /// @param _input which input, inside the epoch, the voucher belongs to
    /// @param _epoch which epoch the voucher belongs to
    /// @return position of that voucher on bitmask
    function getBitMaskPosition(
        uint256 _voucher,
        uint256 _input,
        uint256 _epoch
    ) public pure returns (uint256) {
        // voucher * 2 ** 128 + input * 2 ** 64 + epoch
        // this can't overflow because its impossible to have > 2**128 vouchers
        return (_voucher << 128) + (_input << 64) + _epoch;
    }

    /// @notice get number of finalized epochs
    function getNumberOfFinalizedEpochs()
        public
        view
        override
        returns (uint256)
    {
        return epochHashes.length;
    }
}
