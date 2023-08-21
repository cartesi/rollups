local arithmetic = require "utils.arithmetic"

local log2_uarch_span = 16
local log2_emulator_span = 47

local constants = {
    levels = 3,
    log2step = { 31, 16, 0 },
    heights = { 32, 15, 16 },

    log2_uarch_span = log2_uarch_span,
    uarch_span = arithmetic.max_uint(log2_uarch_span),

    log2_emulator_span = log2_emulator_span,
    emulator_span = arithmetic.max_uint(log2_emulator_span),
}

return constants
