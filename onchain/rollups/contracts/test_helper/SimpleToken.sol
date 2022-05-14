// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title A Simple Token
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract SimpleToken is ERC20 {
    // name: SimpleToken
    // symbol: SIM
    constructor(uint256 initialSupply) ERC20("SimpleToken", "SIM") {
        // on Hardhat network, 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 should be the address of signers[0]
        // generated from default mnemonic "test test test test test test test test test test test junk"
        _mint(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266, initialSupply);
    }
}
