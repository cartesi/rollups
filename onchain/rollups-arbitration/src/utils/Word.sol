// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "./Memory.sol";

library Word {

    //
    // Value
    //

    type Value is bytes8;

    using Word for Value;

    function unwrap(Value v) internal pure returns (bytes8) {
        return Value.unwrap(v);
    }

    function and(Value v1, Value v2) internal pure returns (Value) {
        return Value.wrap(v1.unwrap() & v2.unwrap());
    }

    function or(Value v1, Value v2) internal pure returns (Value) {
        return Value.wrap(v1.unwrap() | Value.unwrap(v2));
    }

    function hash(Value v) internal pure returns (bytes32) {
        return keccak256(abi.encode(v.unwrap()));
    }

    //
    // Slot
    //

    struct Slot {
        Value value;
        Memory.Address memoryAddress;
    }

    function updateValue(Slot memory slot, Value newValue)
        internal pure returns (Slot memory)
    {
        return Slot(newValue, slot.memoryAddress);
    }
}
