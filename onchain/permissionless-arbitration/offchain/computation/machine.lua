local Hash = require "cryptography.hash"
local arithmetic = require "utils.arithmetic"
local cartesi = require "cartesi"

local ComputationState = {}
ComputationState.__index = ComputationState

function ComputationState:new(root_hash, halted, uhalted)
    local r = {
        root_hash = root_hash,
        halted = halted,
        uhalted = uhalted
    }
    setmetatable(r, self)
    return r
end

function ComputationState:from_current_machine_state(machine)
    local hash = Hash:from_digest(machine:get_root_hash())
    return ComputationState:new(
        hash,
        machine:read_iflags_H(),
        machine:read_uarch_halt_flag()
    )
end

ComputationState.__tostring = function(x)
    return string.format(
        "{root_hash = %s, halted = %s, uhalted = %s}",
        x.root_hash,
        x.halted,
        x.uhalted
    )
end


--
---
--

-- TODO Consider removing this abstraction
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
        initial_hash = Hash:from_digest(machine:get_root_hash())
    }

    setmetatable(b, self)
    return b
end

function Machine:state()
    return ComputationState:from_current_machine_state(self.machine)
end

local function add_and_clamp(x, y)
    if math.ult(x, arithmetic.max_uint64 - y) then
        return x + y
    else
        return arithmetic.max_uint64
    end
end

function Machine:run(cycle)
    assert(arithmetic.ulte(self.cycle, cycle))
    local physical_cycle = add_and_clamp(self.start_cycle, cycle) -- TODO reconsider for lambda

    local machine = self.machine
    while not (machine:read_iflags_H() or machine:read_mcycle() == physical_cycle) do
        machine:run(physical_cycle)
    end

    self.cycle = cycle
end

function Machine:run_uarch(ucycle)
    assert(arithmetic.ulte(self.ucycle, ucycle), string.format("%u, %u", self.ucycle, ucycle))
    self.machine:run_uarch(ucycle)
    self.ucycle = ucycle
end

function Machine:increment_uarch()
    self.machine:run_uarch(self.ucycle + 1)
    self.ucycle = self.ucycle + 1
end

function Machine:ureset()
    assert(self.ucycle == arithmetic.max_uint64)
    self.machine:reset_uarch_state()
    self.cycle = self.cycle + 1
    self.ucycle = 0
end

return Machine
