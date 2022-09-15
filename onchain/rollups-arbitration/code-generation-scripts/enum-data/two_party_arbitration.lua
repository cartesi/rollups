local name = "TwoPartyArbitrationEnum"

local sol_version = "^0.8.13"

local imports = {
    "./src/partition/PartitionEnum.sol",
    "./src/epoch-hash-split/EpochHashSplitEnum.sol",
    "./src/splice/SpliceMachineEnum.sol",
    "./src/memory-manager/MemoryManager.sol",
}

local variants = {
    { name = "InputPartition", typ = "PartitionEnum.T" },
    { name = "EpochHashSplit", typ = "EpochHashSplitEnum.T" },
    -- { name = "OutputsSplice", typ = "SpliceOutputsEnum.T" },
    { name = "MachineSplice", typ = "SpliceMachineEnum.T" },
    { name = "InstructionPartition", typ = "PartitionEnum.T" },
    { name = "ProveMemory", typ = "MemoryManager.Context" },
}

return {
    name = name,
    sol_version = sol_version,
    imports = imports,
    variants = variants,
}
