import { keccak256, defaultAbiCoder } from "ethers/lib/utils";

// Calculate input hash based on
// input: data itself interpreted by L2
// blockNumber: `block.number'
// blockTimestamp: `block.timestamp'
// epochIndex: epoch index
// inputIndex: input index
export const getInputHash = (input: any,
                             sender: string,
                             blockNumber: number,
                             blockTimestamp: number,
                             epochIndex: number,
                             inputIndex: number) => {

    // combine input attributes into one
    const metadata = defaultAbiCoder.encode(
        ["uint", "uint", "uint", "uint", "uint"],
        [sender, blockNumber, blockTimestamp, epochIndex, inputIndex]
    );

    // keccak the metadata and the input
    const keccak_metadata = keccak256(metadata);
    const keccak_input = keccak256(input);

    // combine the two keccaks into one
    const abi_metadata_input = defaultAbiCoder.encode(
        ["uint", "uint"],
        [keccak_metadata, keccak_input]
    );

    // keccak the combined keccaks
    const input_hash = keccak256(abi_metadata_input);

    // return the input hash
    return input_hash;
};
