// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Cartesi DApp
pragma solidity 0.8.13;

import {ICartesiDApp} from "./ICartesiDApp.sol";
import {IConsensus} from "../consensus/IConsensus.sol";
import {OutputHeaders} from "../common/OutputHeaders.sol";
import {LibOutputValidationV1, OutputValidityProofV1} from "../library/LibOutputValidationV1.sol";
import {LibOutputValidationV2, OutputValidityProofV2} from "../library/LibOutputValidationV2.sol";
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
    using LibOutputValidationV1 for OutputValidityProofV1;
    using LibOutputValidationV2 for OutputValidityProofV2;

    bytes32 immutable templateHash;
    mapping(uint256 => uint256) voucherBitmask;
    IConsensus consensus;

    constructor(
        IConsensus _consensus,
        address _owner,
        bytes32 _templateHash
    ) {
        transferOwnership(_owner);
        templateHash = _templateHash;
        consensus = _consensus;
    }

    function executeVoucherV1(
        address _destination,
        bytes calldata _payload,
        bytes calldata _claimQuery,
        OutputValidityProofV1 calldata _v
    ) external override nonReentrant returns (bool) {
        bytes32 epochHash;
        uint256 inputIndex;

        // query the current consensus for the epoch hash
        (epochHash, inputIndex) = getEpochHashAndInputIndex(_claimQuery, _v.epochInputIndex);

        bytes memory encodedVoucher = abi.encode(_destination, _payload);

        // reverts if proof isn't valid
        _v.validateEncodedVoucher(encodedVoucher, epochHash);

        uint256 voucherPosition = LibOutputValidationV1.getBitMaskPosition(
            _v.outputIndex,
            inputIndex
        );

        // check if voucher has been executed
        require(
            !voucherBitmask.getBit(voucherPosition),
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

    function validateNoticeV1(
        bytes calldata _notice,
        bytes calldata _claimQuery,
        OutputValidityProofV1 calldata _v
    ) external view override returns (bool) {
        bytes32 epochHash;

        // query the current consensus for the epoch hash
        (epochHash, ) = getEpochHashAndInputIndex(_claimQuery, _v.epochInputIndex);

        bytes memory encodedNotice = abi.encode(_notice);

        // reverts if proof isn't valid
        _v.validateEncodedNotice(encodedNotice, epochHash);

        return true;
    }

    function executeVoucherV2(
        address _destination,
        bytes calldata _payload,
        bytes calldata _claimQuery,
        OutputValidityProofV2 calldata _v
    ) external override nonReentrant returns (bool) {
        bytes32 epochHash;
        uint256 inputIndex;

        // query the current consensus for the epoch hash
        (epochHash, inputIndex) = getEpochHashAndInputIndex(_claimQuery, _v.epochInputIndex);

        // reverts if proof isn't valid
        _v.validateEncodedOutput(
            abi.encodePacked(OutputHeaders.VOUCHER, _destination, _payload),
            epochHash
        );

        uint256 voucherPosition = LibOutputValidationV1.getBitMaskPosition(
            _v.outputIndex,
            inputIndex
        );

        // check if voucher has been executed
        require(
            !voucherBitmask.getBit(voucherPosition),
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

    function validateNoticeV2(
        bytes calldata _notice,
        bytes calldata _claimQuery,
        OutputValidityProofV2 calldata _v
    ) external view override returns (bool) {
        bytes32 epochHash;

        // query the current consensus for the epoch hash
        (epochHash, ) = getEpochHashAndInputIndex(_claimQuery, _v.epochInputIndex);

        // reverts if proof isn't valid
        _v.validateEncodedOutput(
            abi.encode(OutputHeaders.NOTICE, _notice),
            epochHash
        );

        return true;
    }

    function migrateToConsensus(IConsensus _newConsensus)
        external
        override
        onlyOwner
    {
        consensus = _newConsensus;
        emit NewConsensus(_newConsensus);
    }

    function getTemplateHash() external view override returns (bytes32) {
        return templateHash;
    }

    function getConsensus() external view override returns (IConsensus) {
        return consensus;
    }

    function getEpochHashAndInputIndex(
        bytes calldata _claimQuery,
        uint256 _epochInputIndex
    ) internal view returns (bytes32 epochHash_, uint256 inputIndex_) {
        uint256 epochInputIndex;

        (epochHash_, inputIndex_, epochInputIndex) = consensus.getEpochHash(
            address(this),
            _claimQuery
        );

        require(
            _epochInputIndex == epochInputIndex,
            "epoch input indices don't match"
        );
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
