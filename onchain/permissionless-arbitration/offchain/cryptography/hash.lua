local keccak_bin = require "cartesi".keccak

local function hex_from_bin(bin)
    assert(bin:len() == 32)
    return "0x" .. string.gsub(bin, ".", function(c)
        return string.format("%02x", string.byte(c))
    end)
end

local function keccak(...)
    return hex_from_bin(keccak_bin(...))
end

local internalized_hahes = {}
local iterateds = {}

local Hash = {}
Hash.__index = Hash

function Hash:from_digest(digest_hex)
    assert(type(digest_hex) == "string", digest_hex:len() == 66)

    local x = internalized_hahes[digest_hex]
    if x then return x end

    local h = {digest_hex = digest_hex}
    iterateds[h] = {h}
    setmetatable(h, self)
    internalized_hahes[digest_hex] = h
    return h
end

function Hash:from_digest_bin(digest_bin)
    local digest_hex = hex_from_bin(digest_bin)
    return self:from_digest(digest_hex)
end

function Hash:from_data(data)
    local digest_hex = keccak(data)
    return self:from_digest(digest_hex)
end

function Hash:join(other_hash)
    assert(getmetatable(other_hash) == Hash)
    local digest_hex = keccak(self.digest_hex, other_hash.digest_hex)
    local ret = Hash:from_digest(digest_hex)
    ret.left = self.digest_hex
    ret.right = other_hash.digest_hex
    return ret
end

function Hash:children()
    local left, right= self.left, self.right
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

Hash.__tostring = function (x)
    return x.digest_hex
end

local zero_bytes32 = "0x0000000000000000000000000000000000000000000000000000000000000000"
local zero_hash = Hash:from_digest(zero_bytes32)

Hash.zero = zero_hash

function Hash:is_zero()
    return self == zero_hash
end


function Hash:is_of_type_hash(x)
    return getmetatable(x) == self
end

return Hash
