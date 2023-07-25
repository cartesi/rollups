// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

/// @title Test base contract
pragma solidity ^0.8.8;

import {Test} from "forge-std/Test.sol";

contract TestBase is Test {
    /// @notice Guarantess `addr` is an address that can be mocked
    /// @dev Some addresses are reserved by Forge and should not be mocked
    modifier isMockable(address addr) {
        vm.assume(addr != VM_ADDRESS);
        vm.assume(addr != CONSOLE);
        vm.assume(addr != DEFAULT_SENDER);
        vm.assume(addr != DEFAULT_TEST_CONTRACT);
        vm.assume(addr != MULTICALL3_ADDRESS);
        _;
    }
}
