local Hash = require "cryptography.hash"
local consts = require "constants"

local CommitmentBuilder = {}
CommitmentBuilder.__index = CommitmentBuilder

function CommitmentBuilder:new()
    local c = {}
    setmetatable(c, self)
    return c
end

function CommitmentBuilder:build(_, level)
    local commitment = Hash.zero:iterated_merkle(consts.heights[level])
    return commitment
end

return CommitmentBuilder
