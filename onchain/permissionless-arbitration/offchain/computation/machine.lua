local Hash = require "cryptography.hash"
local arithmetic = require "utils.arithmetic"
local cartesi = require "cartesi"

local ComputationResult = {}
ComputationResult.__index = ComputationResult

function ComputationResult:new(state, halted, uhalted)
    local r = {
        state = state,
        halted = halted,
        uhalted = uhalted
    }
    setmetatable(r, self)
    return r
end

function ComputationResult:from_current_machine_state(machine)
    local hash = Hash:from_digest(machine:get_root_hash())
    return ComputationResult:new(
        hash,
        machine:read_iflags_H(),
        machine:read_uarch_halt_flag()
    )
end

ComputationResult.__tostring = function(x)
    return string.format(
        "{state = %s, halted = %s, uhalted = %s}",
        x.state,
        x.halted,
        x.uhalted
    )
end


--
---
--

local Machine = {}
Machine.__index = Machine

function Machine:new_from_path(path)
    local machine = cartesi.machine(path)
    local start_cycle = machine:read_mcycle()

    -- Machine can never be advanced on the micro arch.
    -- Validators must verify this first
    assert(machine:read_uarch_cycle() == 0)

    local b = {
        machine = machine,
        cycle = 0,
        ucycle = 0,
        start_cycle = start_cycle,
    }

    setmetatable(b, self)
    return b
end

function Machine:result()
    return ComputationResult:from_current_machine_state(self.machine)
end

local function add_and_clamp(x, ...)
    for _,v in ipairs {...} do
        if arithmetic.ulte(x, x + v) then
            x = x + v
        else
            return -1
        end
    end

    return x
end

function Machine:advance(cycle, ...)
    cycle = add_and_clamp(cycle, ...)
    assert(self.cycle <= cycle)
    self.machine:run(add_and_clamp(self.start_cycle, cycle))
    self.cycle = cycle
end

function Machine:uadvance(ucycle)
    assert(arithmetic.ulte(self.ucycle, ucycle), string.format("%u, %u", self.ucycle, ucycle))
    self.machine:run_uarch(ucycle)
    self.ucycle = ucycle
end

function Machine:ureset()
    self.machine:reset_uarch_state()
    self.cycle = self.cycle + 1
    self.ucycle = 0
end

return Machine
