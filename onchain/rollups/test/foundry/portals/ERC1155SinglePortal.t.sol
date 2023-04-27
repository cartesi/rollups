// Copyright 2023 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-1155 Single Transfer Portal Test
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";
import {ERC1155SinglePortal} from "contracts/portals/ERC1155SinglePortal.sol";
import {IERC1155SinglePortal} from "contracts/portals/IERC1155SinglePortal.sol";
import {ERC1155} from "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import {IERC1155} from "@openzeppelin/contracts/token/ERC1155/IERC1155.sol";
import {IERC1155Receiver} from "@openzeppelin/contracts/token/ERC1155/IERC1155Receiver.sol";
import {IInputBox} from "contracts/inputs/IInputBox.sol";
import {InputBox} from "contracts/inputs/InputBox.sol";

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

contract ERC1155SinglePortalTest is Test {
    IInputBox inputBox;
    IERC1155SinglePortal portal;
    IERC1155 token;
    address alice;
    address dapp;
    address bob;

    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 value
    );

    function setUp() public {
        inputBox = new InputBox();
        portal = new ERC1155SinglePortal(inputBox);
        alice = vm.addr(1);
        dapp = vm.addr(2);
        bob = vm.addr(3);
    }

    function testGetInputBox() public {
        assertEq(address(portal.getInputBox()), address(inputBox));
    }

    function testERC1155DepositEOA(
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) public {
        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(portal), true);

        // Expect TransferSingle to be emitted with the right arguments
        vm.expectEmit(true, true, true, true);
        emit TransferSingle(address(portal), alice, dapp, tokenId, value);

        portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            baseLayerData,
            execLayerData
        );
        vm.stopPrank();

        // Check the DApp's balance of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testNoBalanceERC1155DepositEOA(
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) public {
        // We can always transfer 0 tokens
        vm.assume(value > 0);

        // Mint ERC1155 tokens for 3rd actor instead of Alice
        token = new NormalToken(bob, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(portal), true);

        vm.expectRevert("ERC1155: insufficient balance for transfer");
        portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            baseLayerData,
            execLayerData
        );
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) public {
        // Use an ERC1155 Receiver contract as a destination
        dapp = address(new ERC1155Receiver());

        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw tokens from Alice
        token.setApprovalForAll(address(portal), true);

        // Expect TransferSingle to be emitted with the right arguments
        vm.expectEmit(true, true, true, true);
        emit TransferSingle(address(portal), alice, dapp, tokenId, value);

        portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            baseLayerData,
            execLayerData
        );
        vm.stopPrank();

        // Check the DApp's balance of the token
        assertEq(token.balanceOf(dapp, tokenId), value);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testNotReceiverERC1155DepositContract(
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) public {
        // Use a contract as a destination that does NOT implement ERC1155 Receiver
        dapp = address(new BadERC1155Receiver());

        // Mint ERC1155 tokens for Alice
        token = new NormalToken(alice, tokenId, value);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.setApprovalForAll(address(portal), true);

        vm.expectRevert("ERC1155: transfer to non-ERC1155Receiver implementer");
        portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            value,
            baseLayerData,
            execLayerData
        );
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }
}

contract ERC1155PortalHandler is Test {
    IERC1155SinglePortal portal;
    address[] dapps;
    IERC1155 token;
    IInputBox inputBox;
    address alice;
    uint256[] tokenIds;
    mapping(uint256 => uint256) public aliceBalances; // tokenId => balance
    mapping(address => mapping(uint256 => uint256)) public dappBalances; // dapp => tokenId => balance
    mapping(address => uint256) public dappNumInputs; // dapp => #inputs

    constructor(
        IERC1155SinglePortal _portal,
        address[] memory _dapps,
        IERC1155 _token,
        uint256[] memory _tokenIds,
        address _alice
    ) {
        portal = _portal;
        dapps = _dapps;
        token = _token;
        inputBox = portal.getInputBox();
        alice = _alice;
        tokenIds = _tokenIds;
        for (uint256 i; i < tokenIds.length; ++i) {
            uint256 tokenId = tokenIds[i];
            aliceBalances[tokenId] = token.balanceOf(alice, tokenId);
        }
    }

    function depositSingleERC1155Token(
        uint256 _dappIndex,
        uint256 _tokenId,
        uint256 _value,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external {
        address dapp = dapps[_dappIndex % dapps.length];
        uint256 tokenId = _tokenId % tokenIds.length;

        if (token.balanceOf(alice, tokenId) == 0) {
            _value = 0;
        } else {
            _value = bound(_value, 0, aliceBalances[tokenId]);
        }

        // check portal balance before tx
        assertEq(token.balanceOf(address(portal), tokenId), 0);
        assertEq(token.balanceOf(alice, tokenId), aliceBalances[tokenId]);
        assertEq(token.balanceOf(dapp, tokenId), dappBalances[dapp][tokenId]);
        assertEq(inputBox.getNumberOfInputs(dapp), dappNumInputs[dapp]);

        vm.startPrank(alice);
        token.setApprovalForAll(address(portal), true);
        portal.depositSingleERC1155Token(
            token,
            dapp,
            tokenId,
            _value,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        aliceBalances[tokenId] -= _value;
        dappBalances[dapp][tokenId] += _value;
        // check balance after tx
        assertEq(token.balanceOf(alice, tokenId), aliceBalances[tokenId]);
        assertEq(token.balanceOf(dapp, tokenId), dappBalances[dapp][tokenId]);
        assertEq(token.balanceOf(address(portal), tokenId), 0);

        assertEq(inputBox.getNumberOfInputs(dapp), ++dappNumInputs[dapp]);
    }
}

contract ERC1155SinglePortalInvariantTest is Test {
    IInputBox inputBox;
    IERC1155SinglePortal portal;
    IERC1155 token;
    ERC1155PortalHandler handler;
    address alice;
    uint256 numTokenIds;
    uint256 numDapps;
    address[] dapps;
    uint256[] tokenIds;
    uint256[] values;

    function setUp() public {
        inputBox = new InputBox();
        portal = new ERC1155SinglePortal(inputBox);
        numDapps = 10;
        // generate dapps/receivers
        for (uint256 i; i < numDapps; ++i) {
            dapps.push(address(new ERC1155Receiver()));
        }
        // batch generate tokens
        numTokenIds = 5;
        for (uint256 i; i < numTokenIds; ++i) {
            tokenIds.push(i);
            values.push(1000000);
        }
        alice = vm.addr(1);
        token = new BatchToken(alice, tokenIds, values);
        handler = new ERC1155PortalHandler(
            portal,
            dapps,
            token,
            tokenIds,
            alice
        );

        targetContract(address(handler));
    }

    function invariantTests() external {
        for (uint256 i; i < numTokenIds; ++i) {
            // check balance for alice
            assertEq(
                token.balanceOf(alice, tokenIds[i]),
                handler.aliceBalances(tokenIds[i])
            );
            for (uint256 j; j < numDapps; ++j) {
                address dapp = dapps[j];
                // check balance for dapp
                assertEq(
                    token.balanceOf(dapp, tokenIds[i]),
                    handler.dappBalances(dapp, tokenIds[i])
                );
            }
            // check balance for portal
            assertEq(token.balanceOf(address(portal), tokenIds[i]), 0);
        }
        // check #inputs
        for (uint256 i; i < numDapps; ++i) {
            address dapp = dapps[i];
            uint256 numInputs = inputBox.getNumberOfInputs(dapp);
            assertEq(numInputs, handler.dappNumInputs(dapp));
        }
    }
}
