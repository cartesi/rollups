// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { Memory } from "utils/Memory.sol";

interface SpliceDataSource {
    struct AddressSpace {
        Memory.Log2Size rxBufferLog2Size;
        Memory.Address rxBufferAddress;

        Memory.Log2Size inputMetadataLog2Size;
        Memory.Address inputMetadataAddress;

        Memory.Log2Size outputHashesLog2Size;
        Memory.Address outputHashesAddress;

        Memory.Address iflagsAddress;
    }

    function getAddressSpace() external pure returns(AddressSpace memory);

    function getInputHash(uint256 epochIndex, uint256 inputIndex)
        external pure returns(bytes32);
    //get hash on concatenation and hash of input and input metadata
//     function getRxBufferLog2Size() external pure returns(uint64);
//     function getRxBufferAddress() external pure returns(uint64);

//     function getInputMetadataLog2Size() external pure returns(uint64);
//     function getInputMetadataAddress() external pure returns(uint64);

//     function getOutputHashesLog2Size() external pure returns(uint64);
//     function getOutputHashesAddress() external pure returns(uint64);

//     function getIflagsAddress() external pure returns(uint64);
}

    /*
    1- Change rx-buffer for next input. Need to merkleize.
    2- Change Input metadata for next input. Need to merkleize.
    3- Zero output hashes. Need to merkleize a "zero tree".
    4- Reset iflags. During operation, will get the word itself, rather than the hash. Need to hash.
   */

