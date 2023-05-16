// Copyright Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title ERC-721 Portal Test
pragma solidity ^0.8.8;

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
    constructor(
        address _tokenOwner,
        uint256 _tokenId
    ) ERC721("NormalToken", "NORMAL") {
        _safeMint(_tokenOwner, _tokenId);
    }

    function mint(address _tokenOwner, uint256 _tokenId) public {
        _safeMint(_tokenOwner, _tokenId);
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
        bytes baseLayerData,
        uint256 numberOfInputs
    );

    constructor(IInputBox _inputBox) {
        inputBox = _inputBox;
    }

    function onERC721Received(
        address _operator,
        address _from,
        uint256 _tokenId,
        bytes calldata _baseLayerData
    ) external override returns (bytes4) {
        uint256 numberOfInputs = inputBox.getNumberOfInputs(address(this));
        emit WatchedTransfer(
            _operator,
            _from,
            _tokenId,
            _baseLayerData,
            numberOfInputs
        );
        return this.onERC721Received.selector;
    }
}

contract ERC721PortalTest is Test {
    IInputBox inputBox;
    IERC721Portal portal;
    IERC721 token;
    address alice;
    address dapp;

    event InputAdded(
        address indexed dapp,
        uint256 indexed inboxInputIndex,
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
        bytes baseLayerData,
        uint256 numberOfInputs
    );

    function setUp() public {
        inputBox = new InputBox();
        portal = new ERC721Portal(inputBox);
        alice = vm.addr(1);
    }

    function testGetInputBox() public {
        assertEq(address(portal.getInputBox()), address(inputBox));
    }

    function testERC721DepositEOA(
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) public {
        // Assume the DApp is an EOA
        dapp = vm.addr(2);

        // Create a normal token with one NFT
        token = new NormalToken(alice, _tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            token,
            alice,
            _tokenId,
            abi.encode(_baseLayerData, _execLayerData)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(portal), _tokenId);

        // Check the owner of the token
        assertEq(token.ownerOf(_tokenId), alice);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, _tokenId);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(portal), input);

        // Transfer ERC-721 tokens to the DApp via the portal
        portal.depositERC721Token(
            token,
            dapp,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        // Check the new owner of the token
        assertEq(token.ownerOf(_tokenId), dapp);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testERC721DepositContract(
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) public {
        // Create contract that implements IERC721Receiver
        ERC721Receiver receiver = new ERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, _tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            token,
            alice,
            _tokenId,
            abi.encode(_baseLayerData, _execLayerData)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(portal), _tokenId);

        // Check the owner of the token
        assertEq(token.ownerOf(_tokenId), alice);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, _tokenId);

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(portal), input);

        // Transfer ERC-721 tokens to the DApp via the portal
        portal.depositERC721Token(
            token,
            dapp,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        // Check the new owner of the token
        assertEq(token.ownerOf(_tokenId), dapp);

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 1);
    }

    function testRevertsNoApproval(
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) public {
        // Create contract that implements IERC721Receiver
        ERC721Receiver receiver = new ERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, _tokenId);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Transfer ERC-721 tokens to the DApp via the portal
        vm.expectRevert("ERC721: caller is not token owner or approved");
        portal.depositERC721Token(
            token,
            dapp,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testRevertsNonImplementer(
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) public {
        // Create contract that refuses ERC-721 transfers
        BadERC721Receiver receiver = new BadERC721Receiver();
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, _tokenId);

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(portal), _tokenId);

        // Expect ERC-721 transfer to revert with message
        vm.expectRevert("This contract refuses ERC-721 transfers");
        portal.depositERC721Token(
            token,
            dapp,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), 0);
    }

    function testNumberOfInputs(
        uint256 _tokenId,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) public {
        // Create a contract that records the number of inputs it has received
        WatcherERC721Receiver receiver = new WatcherERC721Receiver(inputBox);
        dapp = address(receiver);

        // Create a normal token with one NFT
        token = new NormalToken(alice, _tokenId);

        // Construct the ERC-721 deposit input
        bytes memory input = abi.encodePacked(
            token,
            alice,
            _tokenId,
            abi.encode(_baseLayerData, _execLayerData)
        );

        // Start impersonating Alice
        vm.startPrank(alice);

        // Allow the portal to withdraw the token from Alice
        token.approve(address(portal), _tokenId);

        // Get number of inputs on the DApp's input box beforehand
        uint256 numberOfInputsBefore = inputBox.getNumberOfInputs(dapp);

        // Expect Transfer to be emitted with the right arguments
        vm.expectEmit(true, true, true, true, address(token));
        emit Transfer(alice, dapp, _tokenId);

        // Expect receiver to emit event with the base layer data
        vm.expectEmit(false, false, false, true, dapp);
        emit WatchedTransfer(
            address(portal),
            alice,
            _tokenId,
            _baseLayerData,
            numberOfInputsBefore
        );

        // Expect InputAdded to be emitted with the right arguments
        vm.expectEmit(true, true, false, true, address(inputBox));
        emit InputAdded(dapp, 0, address(portal), input);

        // Deposit token in DApp's account
        portal.depositERC721Token(
            token,
            dapp,
            _tokenId,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        // Check the DApp's input box
        assertEq(inputBox.getNumberOfInputs(dapp), numberOfInputsBefore + 1);
    }
}

contract ERC721PortalHandler is Test {
    IERC721Portal portal;
    IERC721 token;
    IInputBox inputBox;
    address alice;
    uint256 aliceBalance;
    uint256 numTokenIds;
    address[] dapps;
    mapping(address => uint256) public dappBalances;

    constructor(
        IERC721Portal _portal,
        address[] memory _dapps,
        IERC721 _token,
        uint256 _numTokenIds,
        address _alice
    ) {
        portal = _portal;
        dapps = _dapps;
        token = _token;
        numTokenIds = _numTokenIds;

        inputBox = portal.getInputBox();
        alice = _alice;
        aliceBalance = token.balanceOf(alice);
    }

    function depositERC721Token(
        uint256 _dappIndex,
        uint256 _tokenIndex,
        bytes calldata _baseLayerData,
        bytes calldata _execLayerData
    ) external {
        uint256 tokenID = _tokenIndex % numTokenIds;
        if (token.ownerOf(tokenID) != alice) return;
        address dapp = dapps[_dappIndex % dapps.length];

        assertEq(token.balanceOf(address(portal)), 0);
        assertEq(token.balanceOf(alice), aliceBalance);
        assertEq(token.ownerOf(tokenID), alice);
        assertEq(token.balanceOf(dapp), dappBalances[dapp]);
        assertEq(inputBox.getNumberOfInputs(dapp), dappBalances[dapp]);

        vm.startPrank(alice);
        token.approve(address(portal), tokenID);
        portal.depositERC721Token(
            token,
            dapp,
            tokenID,
            _baseLayerData,
            _execLayerData
        );
        vm.stopPrank();

        assertEq(token.balanceOf(address(portal)), 0);
        assertEq(token.balanceOf(alice), --aliceBalance);
        assertEq(token.ownerOf(tokenID), dapp);
        assertEq(token.balanceOf(dapp), ++dappBalances[dapp]);
        assertEq(inputBox.getNumberOfInputs(dapp), dappBalances[dapp]);

        dapps.push(dapp);
    }
}

contract ERC721PortalInvariantTest is Test {
    IInputBox inputBox;
    IERC721Portal portal;
    NormalToken token;
    ERC721PortalHandler handler;
    address alice;
    uint256 numTokenIds;
    uint256 numDapps;
    address[] dapps;

    function setUp() public {
        inputBox = new InputBox();
        portal = new ERC721Portal(inputBox);
        // create 30 dapps
        numDapps = 30;
        for (uint256 i; i < numDapps; ++i) {
            dapps.push(address(new ERC721Receiver()));
        }
        // mint 10000 erc721 tokens
        alice = vm.addr(1);
        numTokenIds = 10000;
        token = new NormalToken(alice, 0);
        for (uint256 i = 1; i < numTokenIds; ++i) {
            token.mint(alice, i);
        }
        handler = new ERC721PortalHandler(
            portal,
            dapps,
            token,
            numTokenIds,
            alice
        );

        targetContract(address(handler));
    }

    function invariantTests() external {
        for (uint256 i; i < numDapps; ++i) {
            address dapp = dapps[i];
            assertEq(token.balanceOf(dapp), handler.dappBalances(dapp));
            uint256 numInputs = inputBox.getNumberOfInputs(dapp);
            assertEq(numInputs, handler.dappBalances(dapp));
        }
    }
}
