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
pragma solidity 0.8.13;

import {ICartesiDApp} from "./ICartesiDApp.sol";
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

    /// @notice executes voucher
    /// @param _destination address that will execute the payload
    /// @param _payload payload to be executed by destination
    /// @param _claimData claim data to be handed to consensus
    /// @param _v validity proof for this encoded voucher
    /// @return true if voucher was executed successfully
    /// @dev  vouchers can only be executed once
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        bytes calldata _claimData,
        OutputValidityProof calldata _v
    ) external override nonReentrant returns (bool) {
        bytes32 epochHash;
        uint256 inputIndex;

        (epochHash, inputIndex) = getEpochHashAndInputIndex(_claimData, _v);

        // reverts if validity proof doesnt match
        bytes memory encodedVoucher = abi.encode(_destination, _payload);
        LibOutputValidation.isValidVoucherProof(encodedVoucher, epochHash, _v);

        uint256 voucherPosition = LibOutputValidation.getBitMaskPosition(
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

    /// @notice validates notice
    /// @param _notice notice to be verified
    /// @param _claimData claim data to be handed to consensus
    /// @param _v validity proof for this notice
    /// @return true if notice is valid
    function validateNotice(
        bytes calldata _notice,
        bytes calldata _claimData,
        OutputValidityProof calldata _v
    ) external view override returns (bool) {
        bytes32 epochHash;

        (epochHash, ) = getEpochHashAndInputIndex(_claimData, _v);

        // reverts if proof doesnt match
        bytes memory encodedNotice = abi.encode(_notice);
        LibOutputValidation.isValidNoticeProof(encodedNotice, epochHash, _v);

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
        bytes calldata _claimData,
        OutputValidityProof calldata _v
    ) internal view returns (bytes32 epochHash_, uint256 inputIndex_) {
        uint256 epochInputIndex;

        (epochHash_, inputIndex_, epochInputIndex) = consensus.getEpochHash(
            address(this),
            _claimData
        );

        require(
            _v.epochInputIndex == epochInputIndex,
            "epoch input indices don't match"
        );
    }

    receive() external payable {}

    function withdrawEther(address _receiver, uint256 _value) external {
        require(msg.sender == address(this), "only itself");
        (bool sent, ) = _receiver.call{value: _value}("");
        require(sent, "withdrawEther failed");
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
