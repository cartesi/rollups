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
        uint256 totalSupply
    ) ERC1155("NormalToken") {
        _mint(tokenOwner, tokenId, totalSupply, "");
    }
}

contract BatchToken is ERC1155 {
    constructor(
        address tokenOwner,
        uint256[] memory tokenIds,
        uint256[] memory totalSupplies
    ) ERC1155("BatchToken") {
        _mintBatch(tokenOwner, tokenIds, totalSupplies, "");
    }
}

contract BadERC1155Receiver {}

/* Destination contract that manages ERC-1155 transfers */
contract ERC1155Receiver is IERC1155Receiver {
    function onERC1155Received(
        address,
        address,
        uint256,
        uint256,
        bytes calldata
    ) external pure returns (bytes4) {
        return this.onERC1155Received.selector;
    }

    function onERC1155BatchReceived(
        address,
        address,
        uint256[] memory,
        uint256[] memory,
        bytes calldata
    ) external pure returns (bytes4) {
        return this.onERC1155BatchReceived.selector;
    }

    function supportsInterface(
        bytes4 interfaceID
    ) external pure returns (bool) {
        return
            interfaceID == 0x01ffc9a7 || // ERC-165 support
            interfaceID == 0x4e2312e0; // ERC-1155 `ERC1155TokenReceiver`
    }
}

contract ERC1155PortalTest is Test {
    IInputBox inputBox;
    IERC1155Portal erc1155Portal;
    IERC1155 token;
    address alice;
    address dapp;

    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 value
    );

    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );

    function setUp() public {
        inputBox = new InputBox();
        erc1155Portal = new ERC1155Portal(inputBox);
        alice = address(vm.addr(1));
        dapp = address(vm.addr(2));
    }

    function testGetInputBox() public {
        assertEq(address(erc1155Portal.getInputBox()), address(inputBox));
    }

    function testERC1155DepositEOA(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(value > 0);

        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        // Expect TransferSingle to be emitted with the right arguments
        vm.expectEmit(true, true, true, true);
        emit TransferSingle(
            address(erc1155Portal),
            alice,
            dapp,
            tokenId,
            value
        );

        erc1155Portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );

        // Check the DApp's balance of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testFailNoBalanceERC1155DepositEOA(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(value > 0);
        address bob = address(vm.addr(3));

        // Mint ERC1155 tokens for 3rd actor instead of Alice
        token = new NormalToken(bob, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );
        vm.expectRevert("ERC1155: insufficient balance for transfer");

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(value > 0);

        // Use an ERC1155 Receiver contract as a destination
        dapp = address(new ERC1155Receiver());

        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        // Expect TransferSingle to be emitted with the right arguments
        vm.expectEmit(true, true, true, true);
        emit TransferSingle(
            address(erc1155Portal),
            alice,
            dapp,
            tokenId,
            value
        );

        erc1155Portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            L1data,
            L2data
        );

        // Check the DApp's balance of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testFailnotReceiverERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata L1data,
        bytes calldata L2data
    ) public {
        vm.assume(value > 0);

        // Use a contract as a destination that does NOT implement ERC1155 Receiver
        dapp = address(new BadERC1155Receiver());

        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        erc1155Portal.depositSingleERC1155Token(
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

    function testBatchERC1155DepositEOA(
        bytes calldata L1data,
        bytes calldata L2data,
        uint256[] calldata totalSupplies
    ) public {
        vm.assume(totalSupplies.length > 0);
        uint256[] memory tokenIds = generateTokenIDs(totalSupplies);
        uint256[] memory values = generateValues(totalSupplies);

        // Mint ERC1155 tokens for Alice
        token = new BatchToken(alice, tokenIds, totalSupplies);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(erc1155Portal), true);

        // Expect TransferBatch to be emitted with the right arguments
        vm.expectEmit(true, true, true, true);
        emit TransferBatch(
            address(erc1155Portal),
            alice,
            dapp,
            tokenIds,
            values
        );

        erc1155Portal.depositBatchERC1155Token(
            token,
            dapp,
            tokenIds,
            values,
            L1data,
            L2data
        );

        // Check the token balance of each party
        for (uint256 i; i < totalSupplies.length; ++i) {
            assertEq(token.balanceOf(dapp, tokenIds[i]), values[i]);
            assertEq(
                token.balanceOf(alice, tokenIds[i]),
                totalSupplies[i] - values[i]
            );
        }

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testFailLengthMismatchBatchERC1155DepositEOA(
        uint256[] calldata totalSupplies,
        uint256[] calldata differentSupplies
    ) public {
        vm.assume(totalSupplies.length != differentSupplies.length);
        uint256[] memory tokenIds = generateTokenIDs(totalSupplies);
        uint256[] memory values = generateValues(differentSupplies);

        // Mint ERC1155 tokens for Alice
        token = new BatchToken(alice, tokenIds, values);
        vm.expectRevert("ERC1155: ids and amounts length mismatch");
    }

    function testFailNotApprovedBatchERC1155DepositEOA(
        bytes calldata L1data,
        bytes calldata L2data,
        uint256[] calldata totalSupplies
    ) public {
        vm.assume(totalSupplies.length > 0);
        uint256[] memory tokenIds = generateTokenIDs(totalSupplies);
        uint256[] memory values = generateValues(totalSupplies);

        // Mint tokens for Alice
        token = new BatchToken(alice, tokenIds, values);

        // Start impersonating Alice
        vm.startPrank(alice);

        erc1155Portal.depositBatchERC1155Token(
            token,
            dapp,
            tokenIds,
            values,
            L1data,
            L2data
        );
        vm.expectRevert("ERC1155: caller is not token owner or approved");
    }

    // HELPER FUNCTIONS
    function generateTokenIDs(
        uint256[] calldata totalSupplies
    ) internal pure returns (uint256[] memory) {
        uint256[] memory tokenIds = new uint256[](totalSupplies.length);
        for (uint256 i; i < totalSupplies.length; ++i) tokenIds[i] = i;
        return tokenIds;
    }

    function generateValues(
        uint256[] calldata totalSupplies
    ) internal pure returns (uint256[] memory) {
        uint256[] memory values = new uint256[](totalSupplies.length);
        for (uint256 i; i < totalSupplies.length; ++i) {
            uint256 value = uint256(
                keccak256(abi.encodePacked(i, totalSupplies[i]))
            );
            values[i] = (value <= totalSupplies[i]) ? value : totalSupplies[i];
        }
        return values;
    }

    function generateTokenOwners(
        uint256 totalSupply
    ) internal pure returns (address[] memory) {
        address[] memory tokenOwners = new address[](totalSupply);
        for (uint256 i; i < totalSupply; ++i)
            tokenOwners[i] = address(vm.addr(i + 1));
        return tokenOwners;
    }
}
