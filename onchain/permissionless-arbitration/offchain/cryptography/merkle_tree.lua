local MerkleTree = {}
MerkleTree.__index = MerkleTree

function MerkleTree:new(leafs, root_hash, log2size)
    local m = {
        leafs = leafs,
        root_hash = root_hash,
        digest_hex = root_hash.digest_hex,
        log2size = log2size,
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

-- TODO add generate proof.
-- TODO add children??

return MerkleTree
