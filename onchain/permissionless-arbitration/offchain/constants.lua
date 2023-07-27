local arithmetic = require "utils.arithmetic"

local log2_uarch_span = 64
local log2_emulator_span = 63

local constants = {
    levels = 4,
    max_cycle = 63, -- TODO
    log2step = {24, 14, 7, 0},
    heights = {39, 10, 7, 7},

    log2_uarch_span = log2_uarch_span,
    uarch_span = arithmetic.max_uint(log2_uarch_span),

    log2_emulator_span = log2_emulator_span,
    emulator_span = arithmetic.max_uint(log2_emulator_span),
}

return constants
