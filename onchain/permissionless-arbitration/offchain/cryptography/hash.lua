local keccak = require "cartesi".keccak

local function hex_from_bin(bin)
    assert(bin:len() == 32)
    return "0x" .. (bin:gsub('.', function(c)
        return string.format('%02x', string.byte(c))
    end))
end

local function bin_from_hex(hex)
    assert(hex:len() == 66, string.format("%s %d", hex, hex:len()))
    local h = assert(hex:match("0x(%x+)"), hex)
    return (h:gsub('..', function(cc)
        return string.char(tonumber(cc, 16))
    end))
end

local internalized_hahes = {}
local iterateds = {}

local Hash = {}
Hash.__index = Hash

function Hash:from_digest(digest)
    assert(type(digest) == "string", digest:len() == 32)

    local x = internalized_hahes[digest]
    if x then return x end

    local h = { digest = digest }
    iterateds[h] = { h }
    setmetatable(h, self)
    internalized_hahes[digest] = h
    return h
end

function Hash:from_digest_hex(digest_hex)
    assert(type(digest_hex) == "string", digest_hex:len() == 66)
    local digest = bin_from_hex(digest_hex)
    return self:from_digest(digest)
end

function Hash:from_data(data)
    local digest = keccak(data)
    return self:from_digest(digest)
end

function Hash:join(other_hash)
    assert(Hash:is_of_type_hash(other_hash))

    local digest = keccak(self.digest, other_hash.digest)
    local ret = Hash:from_digest(digest)
    ret.left = self
    ret.right = other_hash
    return ret
end

function Hash:children()
    local left, right = self.left, self.right
    if left and right then
        return true, left, right
    else
        return false
    end
end

function Hash:iterated_merkle(level)
    level = level + 1
    local iterated = iterateds[self]

    local ret = iterated[level]
    if ret then return ret end

    local i = #iterated -- at least 1
    local highest_level = iterated[i]
    while i < level do
        highest_level = highest_level:join(highest_level)
        i = i + 1
        iterated[i] = highest_level
    end

    return highest_level
end

function Hash:hex_string()
    return hex_from_bin(self.digest)
end

Hash.__tostring = function(x)
    return hex_from_bin(x.digest)
end

local zero_bytes32 = "0x0000000000000000000000000000000000000000000000000000000000000000"
local zero_hash = Hash:from_digest_hex(zero_bytes32)

Hash.zero = zero_hash

function Hash:is_zero()
    return self == zero_hash
end

function Hash:is_of_type_hash(x)
    return getmetatable(x) == self
end

return Hash
