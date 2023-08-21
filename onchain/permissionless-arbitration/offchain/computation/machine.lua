local Hash = require "cryptography.hash"
local arithmetic = require "utils.arithmetic"
local cartesi = require "cartesi"
local consts = require "constants"

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

local Machine = {}
Machine.__index = Machine

function Machine:new_from_path(path)
    local machine = cartesi.machine(path)
    local start_cycle = machine:read_mcycle()

    -- Machine can never be advanced on the micro arch.
    -- Validators must verify this first
    assert(machine:read_uarch_cycle() == 0)

    local b = {
        path = path,
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
    self.machine:reset_uarch_state()
    self.cycle = self.cycle + 1
    self.ucycle = 0
end

local keccak = require "cartesi".keccak

local function hex_from_bin(bin)
    assert(bin:len() == 32)
    return "0x" .. (bin:gsub('.', function(c)
        return string.format('%02x', string.byte(c))
    end))
end

local function ver(t, p, s)
    local stride = p >> 3
    for k, v in ipairs(s) do
        if (stride >> (k - 1)) % 2 == 0 then
            t = keccak(t, v)
        else
            t = keccak(v, t)
        end
    end

    return t
end


function Machine:get_logs(path, cycle, ucycle)
    local machine = Machine:new_from_path(path)
    machine:run(cycle)
    machine:run_uarch(ucycle)

    if ucycle == consts.uarch_span then
        error "ureset, not implemented"

        machine:run_uarch(consts.uarch_span)
        -- get reset-uarch logs

        return
    end

    local logs = machine.machine:step_uarch { annotations = true, proofs = true }

    local encoded = {}

    for _, a in ipairs(logs.accesses) do
        assert(a.log2_size == 3)
        if a.type == "read" then
            table.insert(encoded, a.read)
        end

        table.insert(encoded, a.proof.target_hash)

        local siblings = arithmetic.array_reverse(a.proof.sibling_hashes)
        for _, h in ipairs(siblings) do
            table.insert(encoded, h)
        end

        assert(ver(a.proof.target_hash, a.address, siblings) == a.proof.root_hash)
    end

    local data = table.concat(encoded)
    local hex_data = "0x" .. (data:gsub('.', function(c)
        return string.format('%02x', string.byte(c))
    end))

    return '"' .. hex_data .. '"'
end

return Machine
