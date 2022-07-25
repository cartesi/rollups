// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp
pragma solidity ^0.8.13;

import {ICartesiDApp} from "./ICartesiDApp.sol";
import {IAuthority} from "../consensus/authority/IAuthority.sol";
import {IHistory} from "../history/IHistory.sol";
import {LibOutputValidation} from "../library/LibOutputValidation.sol";
import {Bitmask} from "@cartesi/util/contracts/Bitmask.sol";

import {ReentrancyGuard} from "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IERC721Receiver} from "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";

contract CartesiDApp is
    ICartesiDApp,
    IERC721Receiver,
    ReentrancyGuard,
    Ownable
{
    using Bitmask for mapping(uint256 => uint256);

    IAuthority consensus;
    // state hash of the cartesi machine at t0
    bytes32 immutable templateHash;
    mapping(uint256 => uint256) voucherBitmask;
    // we use the following mapping as an array to store all histories used
    uint256 historyLength;
    mapping(uint256 => LibOutputValidation.HistoryBound) histories;

    uint256 epoch;

    constructor(
        address _owner,
        address _consensus,
        bytes32 _templateHash
    ) {
        transferOwnership(_owner);
        migrateToConsensus(_consensus);
        templateHash = _templateHash;
    }

    /// @notice executes voucher
    /// @param _destination address that will execute the payload
    /// @param _payload payload to be executed by destination
    /// @param _v validity proof for this encoded voucher
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        LibOutputValidation.OutputValidityProof calldata _v
    ) public override nonReentrant returns (bool) {
        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        bytes memory claimProofs;
        IHistory history = IHistory(getHistoryAddress(_v.epochIndex));
        bytes32 claim = history.getClaim(
            address(this),
            _v.epochIndex,
            claimProofs
        );

        // reverts if validity proof doesnt match
        LibOutputValidation.isValidVoucherProof(encodedVoucher, claim, _v);

        uint256 voucherPosition = LibOutputValidation.getBitMaskPosition(
            _v.outputIndex,
            _v.inputIndex,
            _v.epochIndex
        );

        // check if voucher has been executed
        require(
            voucherBitmask.getBit(voucherPosition),
            "re-execution not allowed"
        );

        // execute voucher
        (bool succ, ) = _destination.call(_payload);

        // if properly executed, mark it as executed and emit event
        if (succ) {
            voucherBitmask.setBit(voucherPosition, true);
            emit VoucherExecuted(voucherPosition);
        }

        return succ;
    }

    /// @notice validates notice
    /// @param _notice notice to be verified
    /// @param _v validity proof for this notice
    /// @return true if notice is valid
    function validateNotice(
        bytes calldata _notice,
        LibOutputValidation.OutputValidityProof calldata _v
    ) public view override returns (bool) {
        bytes memory encodedNotice = abi.encode(_notice);

        bytes memory claimProofs;
        IHistory history = IHistory(getHistoryAddress(_v.epochIndex));
        bytes32 claim = history.getClaim(
            address(this),
            _v.epochIndex,
            claimProofs
        );

        // reverts if validity proof doesnt match
        LibOutputValidation.isValidNoticeProof(encodedNotice, claim, _v);

        return true;
    }

    function migrateToConsensus(address _consensus) public override onlyOwner {
        consensus = IAuthority(_consensus);
        // check if history address changes
        address newHistory = consensus.getHistoryAddress();
        if (
            historyLength > 0 &&
            histories[historyLength - 1].historyAddress != newHistory
        ) {
            // set epoch upper bound for current history before setting the new history address
            histories[historyLength - 1].epochUpperBound = uint64(epoch - 1);
            histories[historyLength++].historyAddress = newHistory;
        } else if (historyLength == 0) {
            histories[historyLength++].historyAddress = newHistory;
        }
        emit NewConsensus(_consensus);
    }

    function finalizeEpoch() public override {
        // only callable by consensus
        require(msg.sender == address(consensus), "only consensus");

        // check if history address changes
        address newHistory = consensus.getHistoryAddress();
        // historyLength is greater than 0
        if (histories[historyLength - 1].historyAddress != newHistory) {
            // set epoch upper bound for current history before setting the new history address
            histories[historyLength - 1].epochUpperBound = uint64(epoch - 1);
            histories[historyLength++].historyAddress = newHistory;
        }

        ++epoch;
    }

    function getEpoch() public view override returns (uint256) {
        return epoch;
    }

    function getHistoryAddress(uint256 _epoch) internal view returns (address) {
        require(historyLength > 0, "no history yet");
        uint256 historyIndex = historyLength - 1;
        while (
            historyIndex > 0 &&
            histories[historyIndex - 1].epochUpperBound >= _epoch
        ) {
            --historyIndex;
        }
        return histories[historyIndex].historyAddress;
    }

    /// @notice Handle the receipt of an NFT
    /// @dev The ERC721 smart contract calls this function on the recipient
    ///      after a `transfer`. This function MAY throw to revert and reject the
    ///      transfer. Return of other than the magic value MUST result in the
    ///      transaction being reverted.
    /// @return this function selector unless throwing
    function onERC721Received(
        address,
        address,
        uint256,
        bytes calldata
    ) external pure override returns (bytes4) {
        return this.onERC721Received.selector;
    }
}
