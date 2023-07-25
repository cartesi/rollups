// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";

import {IERC1155SinglePortal} from "./IERC1155SinglePortal.sol";
import {InputRelay} from "../inputs/InputRelay.sol";
import {IInputBox} from "../inputs/IInputBox.sol";
import {InputEncoding} from "../common/InputEncoding.sol";

/// @title ERC-1155 Single Transfer Portal
///
/// @notice This contract allows anyone to perform single transfers of
/// ERC-1155 tokens to a DApp while informing the off-chain machine.
contract ERC1155SinglePortal is InputRelay, IERC1155SinglePortal {
    /// @notice Constructs the portal.
    /// @param _inputBox The input box used by the portal
    constructor(IInputBox _inputBox) InputRelay(_inputBox) {}

    function depositSingleERC1155Token(
        IERC1155 _token,
        address _dapp,
        uint256 _tokenId,
        uint256 _value,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external override {
        _token.safeTransferFrom(
            msg.sender,
            _dapp,
            _tokenId,
            _value,
            _baseLayerData
        );

        bytes memory input = InputEncoding.encodeSingleERC1155Deposit(
            _token,
            msg.sender,
            _tokenId,
            _value,
            _baseLayerData,
            _execLayerData
        );

        inputBox.addInput(_dapp, input);
    }
}
