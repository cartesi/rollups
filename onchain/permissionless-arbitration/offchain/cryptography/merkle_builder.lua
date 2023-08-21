local MerkleTree = require "cryptography.merkle_tree"
local arithmetic = require "utils.arithmetic"

local ulte = arithmetic.ulte
local semi_sum = arithmetic.semi_sum

local Slice = {}
Slice.__index = Slice

function Slice:new(arr, start_idx_inc, end_idx_ex)
    start_idx_inc = start_idx_inc or 1
    end_idx_ex = end_idx_ex or (#arr + 1)
    assert(start_idx_inc > 0)
    assert(ulte(start_idx_inc, end_idx_ex))
    assert(end_idx_ex <= #arr + 1)
    local s = {
        arr = arr,
        start_idx_inc = start_idx_inc,
        end_idx_ex = end_idx_ex,
    }
    setmetatable(s, self)
    return s
end

function Slice:slice(si, ei)
    assert(si > 0)
    assert(ulte(si, ei))
    local start_idx_inc = self.start_idx_inc + si - 1
    local end_idx_ex = self.start_idx_inc + ei - 1
    assert(ulte(end_idx_ex, self.end_idx_ex))
    return Slice:new(self.arr, start_idx_inc, end_idx_ex)
end

function Slice:len()
    return self.end_idx_ex - self.start_idx_inc
end

function Slice:get(idx)
    idx = assert(math.tointeger(idx))
    assert(idx > 0)
    local i = self.start_idx_inc + idx - 1
    assert(i < self.end_idx_ex)
    return self.arr[i]
end

function Slice:find_cell_containing(elem)
    local l, r = 1, self:len()

    while math.ult(l, r) do
        local m = semi_sum(l, r)

        -- `-1` on both sides changes semantics on underflow... zero means 2^64.
        if math.ult(self:get(m).accumulated_count - 1, elem - 1) then
            l = m + 1
        else
            r = m
        end
    end

    return l
end

local MerkleBuilder = {}
MerkleBuilder.__index = MerkleBuilder

function MerkleBuilder:new()
    local m = {
        leafs = {},
    }
    setmetatable(m, self)
    return m
end

function MerkleBuilder:add(hash, rep)
    rep = rep or 1
    assert(math.ult(0, rep))

    local last = self.leafs[#self.leafs]
    if last then
        assert(last.accumulated_count ~= 0, "merkle builder is full")
        local accumulated_count = rep + last.accumulated_count

        if not math.ult(rep, accumulated_count) then -- overflow...
            assert(accumulated_count == 0)           -- then it has to be zero, and nothing else can fit.
        end

        table.insert(self.leafs, { hash = hash, accumulated_count = accumulated_count })
    else
        table.insert(self.leafs, { hash = hash, accumulated_count = rep })
    end
end

local function merkle(leafs, log2size, stride)
    local first_time = stride * (1 << log2size) + 1
    local last_time = (stride + 1) * (1 << log2size)

    local first_cell = leafs:find_cell_containing(first_time)
    local last_cell = leafs:find_cell_containing(last_time)

    if first_cell == last_cell then
        return leafs:get(first_cell).hash:iterated_merkle(log2size)
    end

    local slice = leafs:slice(first_cell, last_cell + 1)
    local hash_left = merkle(slice, log2size - 1, stride << 1)
    local hash_right = merkle(slice, log2size - 1, (stride << 1) + 1)

    return hash_left:join(hash_right)
end

function MerkleBuilder:build(implicit_hash)
    local last = assert(self.leafs[#self.leafs], #self.leafs)
    local count = last.accumulated_count

    local log2size
    if count == 0 then
        log2size = 64
    else
        assert(arithmetic.is_pow2(count), count)
        log2size = arithmetic.ctz(count)
    end

    local root_hash = merkle(Slice:new(self.leafs), log2size, 0)
    return MerkleTree:new(self.leafs, root_hash, log2size, implicit_hash)
end

-- local Hash = require "cryptography.hash"
-- local builder = MerkleBuilder:new()
-- builder:add(Hash.zero, 2)
-- builder:add(Hash.zero)
-- builder:add(Hash.zero)
-- builder:add(Hash.zero, 3)
-- builder:add(Hash.zero)
-- builder:add(Hash.zero, 0 - 8)
-- print(builder:build().root_hash)

-- print(Hash.zero:iterated_merkle(64))

return MerkleBuilder
