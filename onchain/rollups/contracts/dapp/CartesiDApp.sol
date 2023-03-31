// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp
pragma solidity ^0.8.8;

import {ICartesiDApp, Proof} from "./ICartesiDApp.sol";
import {IConsensus} from "../consensus/IConsensus.sol";
import {LibOutputValidation, OutputValidityProof} from "../library/LibOutputValidation.sol";
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
    using LibOutputValidation for OutputValidityProof;

    bytes32 internal immutable templateHash;
    mapping(uint256 => uint256) internal voucherBitmask;
    IConsensus internal consensus;

    constructor(IConsensus _consensus, address _owner, bytes32 _templateHash) {
        transferOwnership(_owner);
        templateHash = _templateHash;
        consensus = _consensus;

        _consensus.join();
    }

    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        Proof calldata _proof
    ) external override nonReentrant returns (bool) {
        bytes32 epochHash;
        uint256 firstInputIndex;
        uint256 lastInputIndex;
        uint256 inboxInputIndex;

        // query the current consensus for the desired claim
        (epochHash, firstInputIndex, lastInputIndex) = getClaim(_proof.context);

        // validate the epoch input index and calculate the inbox input index
        // based on the input index range provided by the consensus
        inboxInputIndex = _proof.validity.validateInputIndexRange(
            firstInputIndex,
            lastInputIndex
        );

        // reverts if proof isn't valid
        _proof.validity.validateVoucher(_destination, _payload, epochHash);

        uint256 voucherPosition = LibOutputValidation.getBitMaskPosition(
            _proof.validity.outputIndex,
            inboxInputIndex
        );

        // check if voucher has been executed
        require(
            !_wasVoucherExecuted(voucherPosition),
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

    function wasVoucherExecuted(
        uint256 _inboxInputIndex,
        uint256 _outputIndex
    ) external view override returns (bool) {
        uint256 voucherPosition = LibOutputValidation.getBitMaskPosition(
            _outputIndex,
            _inboxInputIndex
        );
        return _wasVoucherExecuted(voucherPosition);
    }

    function _wasVoucherExecuted(
        uint256 _voucherPosition
    ) internal view returns (bool) {
        return voucherBitmask.getBit(_voucherPosition);
    }

    function validateNotice(
        bytes calldata _notice,
        Proof calldata _proof
    ) external view override returns (bool) {
        bytes32 epochHash;
        uint256 firstInputIndex;
        uint256 lastInputIndex;

        // query the current consensus for the desired claim
        (epochHash, firstInputIndex, lastInputIndex) = getClaim(_proof.context);

        // validate the epoch input index based on the input index range
        // provided by the consensus
        _proof.validity.validateInputIndexRange(
            firstInputIndex,
            lastInputIndex
        );

        // reverts if proof isn't valid
        _proof.validity.validateNotice(_notice, epochHash);

        return true;
    }

    function getClaim(
        bytes calldata _proofContext
    ) internal view returns (bytes32, uint256, uint256) {
        return consensus.getClaim(address(this), _proofContext);
    }

    function migrateToConsensus(
        IConsensus _newConsensus
    ) external override onlyOwner {
        consensus = _newConsensus;

        _newConsensus.join();

        emit NewConsensus(_newConsensus);
    }

    function getTemplateHash() external view override returns (bytes32) {
        return templateHash;
    }

    function getConsensus() external view override returns (IConsensus) {
        return consensus;
    }

    receive() external payable {}

    function withdrawEther(address _receiver, uint256 _value) external {
        require(msg.sender == address(this), "only itself");
        (bool sent, ) = _receiver.call{value: _value}("");
        require(sent, "withdrawEther failed");
    }

    function onERC721Received(
        address,
        address,
        uint256,
        bytes calldata
    ) external pure override returns (bytes4) {
        return this.onERC721Received.selector;
    }
}
