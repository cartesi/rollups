local MerkleBuilder = require "cryptography.merkle_builder"
local Hash = require "cryptography.hash"
local consts = require "constants"

local CommitmentBuilder = {}
CommitmentBuilder.__index = CommitmentBuilder

function CommitmentBuilder:new(initial_hash, second_state)
    local c = { initial_hash = initial_hash, second_state = second_state }
    setmetatable(c, self)
    return c
end

function CommitmentBuilder:build(_, level)
    local builder = MerkleBuilder:new()
    if consts.log2step[consts.levels - level + 1] == 0 and self.second_state then
        builder:add(self.second_state)
        builder:add(Hash.zero, (1 << consts.heights[consts.levels - level + 1]) - 1)
    else
        builder:add(Hash.zero, 1 << consts.heights[consts.levels - level + 1])
    end
    -- local commitment = Hash.zero:iterated_merkle(consts.heights[level])
    return builder:build(self.initial_hash)
end

return CommitmentBuilder
