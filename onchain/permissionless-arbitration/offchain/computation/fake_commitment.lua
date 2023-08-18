local MerkleBuilder = require "cryptography.merkle_builder"
local Hash = require "cryptography.hash"
local consts = require "constants"

local CommitmentBuilder = {}
CommitmentBuilder.__index = CommitmentBuilder

function CommitmentBuilder:new(initial_hash)
    local c = { initial_hash = initial_hash }
    setmetatable(c, self)
    return c
end

function CommitmentBuilder:build(_, level)
    local builder = MerkleBuilder:new()
    builder:add(Hash.zero, 1 << consts.heights[consts.levels - level + 1])
    -- local commitment = Hash.zero:iterated_merkle(consts.heights[level])
    return builder:build(self.initial_hash)
end

return CommitmentBuilder
