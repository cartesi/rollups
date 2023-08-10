local function max_uint(k)
    assert(k <= 64)
    return (1 << k) - 1
end

local max_uint64 = max_uint(64)

local function ulte(x, y)
    return x == y or math.ult(x, y)
end

local function is_pow2(x)
    return (x & (x - 1)) == 0
end

-- Returns number of leading zeroes of x. Shamelessly stolen from the book
-- Hacker's Delight.
local function clz(x)
    if x == 0 then return 64 end
    local n = 0
    if (x & 0xFFFFFFFF00000000) == 0 then
        n = n + 32; x = x << 32
    end
    if (x & 0xFFFF000000000000) == 0 then
        n = n + 16; x = x << 16
    end
    if (x & 0xFF00000000000000) == 0 then
        n = n + 8; x = x << 8
    end
    if (x & 0xF000000000000000) == 0 then
        n = n + 4; x = x << 4
    end
    if (x & 0xC000000000000000) == 0 then
        n = n + 2; x = x << 2
    end
    if (x & 0x8000000000000000) == 0 then n = n + 1 end
    return n
end

-- Returns number of trailing zeroes of x. Shamelessly stolen from the book
-- Hacker's Delight.
local function ctz(x)
    x = x & (~x + 1)
    return 63 - clz(x)
end

local function semi_sum(a, b)
    assert(ulte(a, b))
    return a + (b - a) // 2
end

return {
    max_uint = max_uint,
    max_uint64 = max_uint64,
    ulte = ulte,
    is_pow2 = is_pow2,
    clz = clz,
    ctz = ctz,
    semi_sum = semi_sum,
}
