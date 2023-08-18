local MerkleTree = {}
MerkleTree.__index = MerkleTree

function MerkleTree:new(leafs, root_hash, log2size, implicit_hash)
    local m = {
        leafs = leafs,
        root_hash = root_hash,
        digest_hex = root_hash.digest_hex,
        log2size = log2size,
        implicit_hash = implicit_hash
    }
    setmetatable(m, self)
    return m
end

function MerkleTree:join(other_hash)
    return self.root_hash:join(other_hash)
end

function MerkleTree:children()
    return self.root_hash:children()
end

function MerkleTree:iterated_merkle(level)
    return self.root_hash:iterated_merkle(level)
end

function MerkleTree:hex_string()
    return self.root_hash:hex_string()
end

MerkleTree.__tostring = function(x)
    return x.root_hash:hex_string()
end


local function generate_proof(proof, root, height, include_index)
    if height == 0 then
        proof.leaf = root
        return
    end

    local new_height = height - 1
    local ok, left, right = root:children()
    assert(ok)

    if (include_index >> new_height) & 1 == 0 then
        generate_proof(proof, left, new_height, include_index)
        table.insert(proof, right)
    else
        generate_proof(proof, right, new_height, include_index)
        table.insert(proof, left)
    end
end

function MerkleTree:prove_leaf(index)
    local height
    local l = assert(self.leafs[1])
    if l.log2size then
        height = l.log2size + self.log2size
    else
        height = self.log2size
    end

    print(index, height, "P")

    assert((index >> height) == 0)
    local proof = {}
    generate_proof(proof, self.root_hash, height, index)
    return proof.leaf, proof
end

local function array_reverse(x)
    local n, m = #x, #x / 2
    for i = 1, m do
        x[i], x[n - i + 1] = x[n - i + 1], x[i]
    end
    return x
end

function MerkleTree:last()
    local proof = {}
    local ok, left, right = self.root_hash:children()
    local old_right = self.root_hash

    while ok do
        table.insert(proof, left)
        old_right = right
        ok, left, right = right:children()
    end

    return old_right, array_reverse(proof)
end

-- local Hash = require "cryptography.hash"
-- local MerkleBuilder = require "cryptography.merkle_builder"
-- local builder = MerkleBuilder:new()
-- builder:add(Hash.zero, 1 << 8)
-- local mt = builder:build()

-- local i, p = mt:last((1 << 8) - 1)
-- local r = assert(i)
-- print(i)
-- for _, v in ipairs(p) do
--     print(v)
--     r = v:join(r)
-- end

-- print("FINAL", r, mt.root_hash)

return MerkleTree
