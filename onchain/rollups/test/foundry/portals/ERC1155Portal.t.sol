// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Portal Test
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {ERC1155Portal} from "contracts/portals/ERC1155Portal.sol";
import {IERC1155Portal} from "contracts/portals/IERC1155Portal.sol";
import {ERC1155} from "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IERC1155Receiver} from "@openzeppelin/contracts/token/ERC1155/IERC1155Receiver.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";
import {InputEncoding} from "contracts/common/InputEncoding.sol";

contract NormalToken is ERC1155 {
    constructor(
        address tokenOwner,
        uint256 tokenId,
        uint256 value,
        bytes memory data
    ) ERC1155("NormalToken") {
        _mint(tokenOwner, tokenId, value, data);
    }
}

contract BatchToken is ERC1155 {
    constructor(
        address tokenOwner,
        uint256[] memory tokenIds,
        uint256[] memory values,
        bytes memory data
    ) ERC1155("BatchToken") {
        _mintBatch(tokenOwner, tokenIds, values, data);
    }
}

contract BadERC1155Receiver {}

/* Destination cntract that manages ERC-1155 Transfers */
contract ERC1155Receiver is IERC1155Receiver {
    function onERC1155Received(
        address,
        address,
        uint256,
        uint256,
        bytes calldata
    ) external pure returns (bytes4) {
        return
            bytes4(
                keccak256(
                    "onERC1155Received(address,address,uint256,uint256,bytes)"
                )
            );
    }

    function onERC1155BatchReceived(
        address,
        address,
        uint256[] memory,
        uint256[] memory,
        bytes calldata
    ) external pure returns (bytes4) {
        return
            bytes4(
                keccak256(
                    "onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"
                )
            );
    }

    function supportsInterface(
        bytes4 interfaceID
    ) external pure returns (bool) {
        return
            interfaceID == 0x01ffc9a7 || // ERC-165 support (i.e. `bytes4(keccak256('supportsInterface(bytes4)'))`).
            interfaceID == 0x4e2312e0; // ERC-1155 `ERC1155TokenReceiver` support (i.e. `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)")) ^ bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"))`).
    }
}

contract ERC1155PortalTest is Test {
    IInputBox inputBox;
    IERC1155Portal erc1155Portal;
    IERC1155 token;
    address alice;
    address dapp;

    function setUp() public {
        inputBox = new InputBox();
        erc1155Portal = new ERC1155Portal(inputBox);
        alice = address(vm.addr(1));
        dapp = address(vm.addr(2));
    }

    function testGetInputBox() public {
        assertEq(address(erc1155Portal.getInputBox()), address(inputBox));
    }

    function testERC1155DepositTrue(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(tokenId != 0);
        vm.assume(value != 0);
        //Mint 1155 Tokens for Alice
        token = new NormalToken(alice, tokenId, value, L1data);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );

        // Check the new owner of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testERC1155DepositFalse(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(tokenId != 0);
        vm.assume(value != 0);
        address bob = address(vm.addr(3));

        //Mint 1155 Tokens for 3rd actor instead of Alice
        token = new NormalToken(bob, tokenId, value, L1data);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        vm.expectRevert("ERC1155: insufficient balance for transfer");
        erc1155Portal.depositERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(tokenId != 0);
        vm.assume(value != 0);
        //Use a contract as a destination that does not implement ERC1155 Receiver
        ERC1155Receiver receiver = new ERC1155Receiver();
        dapp = address(receiver);

        //Mint 1155 Tokens for Alice
        token = new NormalToken(alice, tokenId, value, L1data);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );

        // Check the new owner of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testFailERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        //Use a contract as a destination that does not implement ERC1155 Receiver
        BadERC1155Receiver receiver = new BadERC1155Receiver();
        dapp = address(receiver);

        //Mint 1155 Tokens for Alice
        token = new NormalToken(alice, tokenId, value, L1data);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );
        vm.expectRevert("ERC1155: transfer to non-ERC1155Receiver implementer");

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testBatchERC1155DepositTrue(
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        address[] memory tokenOwners = new address[](5);
        uint256[] memory tokenIds = new uint256[](5);
        uint256[] memory values = new uint256[](5);

        for (uint8 i = 0; i < tokenIds.length; ++i) {
            tokenIds[i] = i;
            values[i] = i << 2;
            tokenOwners[i] = vm.addr(i + 2); //as two previous addresses are already in use
        }
        tokenOwners[0] = alice;
        tokenOwners[1] = dapp;

        //Mint 1155 Tokens for Alice
        token = new BatchToken(alice, tokenIds, values, L1data);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositBatchERC1155Token(
            token,
            dapp,
            tokenIds,
            values,
            L1data,
            L2data
        );

        //Owner[1] is dapp address and has 4 different tipes of tokenId, not 5 as the amount of tokenId[0] = 0
        assertEq(token.balanceOfBatch(tokenOwners, tokenIds)[1], 4);

        //Owner[] is alice address and has no tokens
        assertEq(token.balanceOfBatch(tokenOwners, tokenIds)[0], 0);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testBatchERC1155DepositFalse(bytes calldata L1data) public {
        address[] memory tokenOwners = new address[](5);
        uint256[] memory tokenIds = new uint256[](4);
        uint256[] memory values = new uint256[](5);

        for (uint8 i = 0; i < tokenIds.length; ++i) {
            tokenIds[i] = i;
            values[i] = i << 2;
            tokenOwners[i] = vm.addr(i + 2); //two previous addresses are already in use
        }
        tokenOwners[0] = alice;
        tokenOwners[1] = dapp;

        // Mint 1155 Tokens for Alice
        // As tokendIds and values has different lenghts, batching will fail
        vm.expectRevert("ERC1155: ids and amounts length mismatch");
        token = new BatchToken(alice, tokenIds, values, L1data);
    }
}
