// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-721 Portal Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {ERC721Portal} from "contracts/portals/ERC721Portal.sol";
import {IERC721Portal} from "contracts/portals/IERC721Portal.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {IERC721Receiver} from "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputEncoding} from "contracts/common/InputEncoding.sol";

contract NormalToken is ERC721 {
    constructor(address tokenOwner, uint256 tokenId)
        ERC721("NormalToken", "NORMAL")
    {
        _safeMint(tokenOwner, tokenId);
    }
}

contract ERC721Receiver is IERC721Receiver {
    function onERC721Received(
        address,
        address,
        uint256,
        bytes calldata
    ) external pure override returns (bytes4) {
        return this.onERC721Received.selector;
    }
}

contract BadERC721Receiver is IERC721Receiver {
    function onERC721Received(
        address,
        address,
        uint256,
        bytes calldata
    ) external pure override returns (bytes4) {
        revert("This contract refuses ERC-721 transfers");
    }
}

contract WatcherERC721Receiver is IERC721Receiver {
    IInputBox inputBox;

    event WatchedTransfer(
        address operator,
        address from,
        uint256 tokenId,
        bytes L1data,
        uint256 numberOfInputs
    );

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function onERC721Received(
        address _operator,
        address _from,
        uint256 _tokenId,
        bytes calldata _L1data
    ) external override returns (bytes4) {
        uint256 numberOfInputs = inputBox.getNumberOfInputs(address(this));
        emit WatchedTransfer(
            _operator,
            _from,
            _tokenId,
            _L1data,
            numberOfInputs
        );
        return this.onERC721Received.selector;
    }
}

contract ERC721PortalTest is Test {
    IInputBox inputBox;
    IERC721Portal erc721Portal;
    IERC721 token;
    address alice;
    address dapp;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inputIndex,
        address sender,
        bytes input
    );
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 indexed tokenId
    );
    event WatchedTransfer(
        address operator,
        address from,
        uint256 tokenId,
        bytes L1data,
        uint256 numberOfInputs
    );

    function setUp() public {
        inputBox = new InputBox();
        erc721Portal = new ERC721Portal(inputBox);
        alice = address(0xdeadbeef);
    }

    function testGetInputBox() public {
        assertEq(address(erc721Portal.getInputBox()), address(inputBox));
    }

    function testERC721DepositEOA(
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        // Assume the DApp is an EOA
        dapp = address(0x12345678);

        // Create a normal token with one NFT
        token = new NormalToken(alice, tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            InputEncoding.ERC721_DEPOSIT,
            token,
            alice,
            tokenId,
            abi.encode(L1data, L2data)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(erc721Portal), tokenId);

        // Check the owner of the token
        assertEq(token.ownerOf(tokenId), alice);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, tokenId);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(erc721Portal), input);

        // Transfer ERC-721 tokens to the DApp via the portal
        erc721Portal.depositERC721Token(token, dapp, tokenId, L1data, L2data);

        // Check the new owner of the token
        assertEq(token.ownerOf(tokenId), dapp);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testERC721DepositContract(
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        // Create contract that implements IERC721Receiver
        ERC721Receiver receiver = new ERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            InputEncoding.ERC721_DEPOSIT,
            token,
            alice,
            tokenId,
            abi.encode(L1data, L2data)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(erc721Portal), tokenId);

        // Check the owner of the token
        assertEq(token.ownerOf(tokenId), alice);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, tokenId);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(erc721Portal), input);

        // Transfer ERC-721 tokens to the DApp via the portal
        erc721Portal.depositERC721Token(token, dapp, tokenId, L1data, L2data);

        // Check the new owner of the token
        assertEq(token.ownerOf(tokenId), dapp);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsNoApproval(
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        // Create contract that implements IERC721Receiver
        ERC721Receiver receiver = new ERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, tokenId);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Transfer ERC-721 tokens to the DApp via the portal
        vm.expectRevert("ERC721: transfer caller is not owner nor approved");
        erc721Portal.depositERC721Token(token, dapp, tokenId, L1data, L2data);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testRevertsNonImplementer(
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        // Create contract that refuses ERC-721 transfers
        BadERC721Receiver receiver = new BadERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, tokenId);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(erc721Portal), tokenId);

        // Expect ERC-721 transfer to revert with message
        vm.expectRevert("This contract refuses ERC-721 transfers");
        erc721Portal.depositERC721Token(token, dapp, tokenId, L1data, L2data);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testNumberOfInputs(
        uint256 tokenId,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        // Create a contract that records the number of inputs it has received
        WatcherERC721Receiver receiver = new WatcherERC721Receiver(inputBox);
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            InputEncoding.ERC721_DEPOSIT,
            token,
            alice,
            tokenId,
            abi.encode(L1data, L2data)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(erc721Portal), tokenId);

        // Get number of inputs on the DApp's input box beforehand
        uint256 numberOfInputsBefore = inputBox.getNumberOfInputs(dapp);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, tokenId);

        // Expect receiver to emit event with L1 data
        vm.expectEmit(false, false, false, true, dapp);
        emit WatchedTransfer(
            address(erc721Portal),
            alice,
            tokenId,
            L1data,
            numberOfInputsBefore
        );

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, false, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(erc721Portal), input);

        // Deposit token in DApp's account
        erc721Portal.depositERC721Token(token, dapp, tokenId, L1data, L2data);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), numberOfInputsBefore + 1);
    }
}
