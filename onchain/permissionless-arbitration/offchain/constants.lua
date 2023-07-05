local constants = {
    levels = 4,
    max_cycle = 63,
    log2step = {24, 14, 7, 0},
    heights = {39, 10, 7, 7},
    a = 64, b = 63,
}

--[[
a = 2
b = 2

states = 2^b + 1

x (0 0 0 | x) (0 0 0 | x) (0 0 0 | x) (0 0 0 | x)

0  1 2 3   0   1 2 3   0   0

--]]

return constants
