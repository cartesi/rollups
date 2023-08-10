local MerkleBuilder = require "cryptography.merkle_builder"
local Machine = require "computation.machine"

local arithmetic = require "utils.arithmetic"
local consts = require "constants"

local ulte = arithmetic.ulte

local function run_uarch_span(machine)
    assert(machine.ucycle == 0)
    machine:increment_uarch()
    local builder = MerkleBuilder:new()

    local i = 0
    repeat
        builder:add(machine:state().root_hash)
        machine:increment_uarch()
        i = i + 1
    until machine:state().uhalted

    -- Add all remaining fixed-point states, filling the tree up to the last leaf.
    builder:add(machine:state().root_hash, consts.uarch_span - i)

    -- At this point, we've added `2^a - 1` hashes to the inner merkle builder.
    -- Note that these states range from "meta" ucycle `1` to `2^a - 1`.

    -- Now we do the last state transition (ureset), and add the last state,
    -- closing in a power-of-two number of leaves (`2^a` leaves).
    machine:ureset()
    builder:add(machine:state().root_hash)

    return builder:build()
end

local function build_small_machine_commitment(base_cycle, log2_stride_count, machine)
    machine:run(base_cycle)
    local initial_state = machine:state().root_hash

    local builder = MerkleBuilder:new()
    local instruction_count = arithmetic.max_uint(log2_stride_count - consts.log2_uarch_span)
    local instruction = 0
    while ulte(instruction, instruction_count) do
        builder:add(run_uarch_span(machine))
        instruction = instruction + 1

        -- Optional optimization, just comment to remove.
        if machine:state().halted then
            builder:add(run_uarch_span(machine), instruction_count - instruction + 1)
            break
        end
    end

    return initial_state, builder:build()
end



local function build_big_machine_commitment(base_cycle, log2_stride, log2_stride_count, machine)
    machine:run(base_cycle)
    local initial_state = machine:state().root_hash

    local builder = MerkleBuilder:new()
    local instruction_count = arithmetic.max_uint(log2_stride_count)
    local instruction = 0
    while ulte(instruction, instruction_count) do
        local cycle = ((instruction + 1) << (log2_stride - consts.log2_uarch_span))
        machine:run(base_cycle + cycle)

        if not machine:state().halted then
            builder:add(machine:state().root_hash)
            instruction = instruction + 1
        else
            -- add this loop plus all remainings
            builder:add(machine:state().root_hash, instruction_count - instruction + 1)
            break
        end
    end

    return initial_state, builder:build()
end

local function build_commitment(base_cycle, log2_stride, log2_stride_count, machine_path)
    local machine = Machine:new_from_path(machine_path)

    if log2_stride >= consts.log2_uarch_span then
        assert(
            log2_stride + log2_stride_count <=
            consts.log2_emulator_span + consts.log2_uarch_span
        )
        return build_big_machine_commitment(base_cycle, log2_stride, log2_stride_count, machine)
    else
        assert(log2_stride == 0)
        return build_small_machine_commitment(base_cycle, log2_stride_count, machine)
    end
end

local CommitmentBuilder = {}
CommitmentBuilder.__index = CommitmentBuilder

function CommitmentBuilder:new(machine_path)
    local c = {
        machine_path = machine_path,
        commitments = {}
    }
    setmetatable(c, self)
    return c
end

function CommitmentBuilder:build(base_cycle, level)
    assert(level <= consts.levels)
    if not self.commitments[level] then
        self.commitments[level] = {}
    elseif self.commitments[level][base_cycle] then
        return self.commitments[level][base_cycle]
    end

    local l = consts.levels - level + 1
    local log2_stride, log2_stride_count = consts.log2step[l], consts.heights[l]
    print(log2_stride, log2_stride_count)

    local _, commitment = build_commitment(base_cycle, log2_stride, log2_stride_count, self.machine_path)
    self.commitments[level][base_cycle] = commitment
    return commitment
end

-- local path = "program/simple-program"
-- -- local initial, tree = build_commitment(0, 0, 64, path)
-- local initial, tree = build_commitment(400, 0, 67, path)
-- local initial, tree = build_commitment(0, 64, 63, path)
-- print(initial, tree.root_hash)

-- 0x95ebed36f6708365e01abbec609b89e5b2909b7a127636886afeeffafaf0c2ec
-- 0x0f42278e1dd53a54a4743633bcbc3db7035fd9952eccf5fcad497b6f73c8917c
--
--0xd4a3511d1c56eb421e64dc218e8d7bf29c5d3ad848306f04c1b7f43b8883b670
--0x66af9174ab9acb9d47d036b2e735cb9ba31226fd9b06198ce5bc0782c5ca03ff
--
-- 0x95ebed36f6708365e01abbec609b89e5b2909b7a127636886afeeffafaf0c2ec
-- 0xa27e413a85c252c5664624e5a53c5415148b443983d7101bb3ca88829d1ab269


--[[
--[[
a = 2
b = 2

states = 2^b + 1

x (0 0 0 | x) (0 0 0 | x) (0 0 0 | x) (0 0 0 | x)
0  1 2 3   0   1 2 3   0   1
--]]




-- local function x(log2_stride, log2_stride_count, machine)
--     local uarch_instruction_count = arithmetic.max_uint(log2_stride_count)
--     local stride = 1 << log2_stride
--     local inner_builder = MerkleBuilder:new()

--     local ucycle = stride
--     while ulte(ucycle, uarch_instruction_count) do
--         machine:run_uarch(ucycle)
--         local state = machine:state()

--         if not state.uhalted then
--             inner_builder:add(state.state)
--             ucycle = ucycle + stride
--         else
--             -- add this loop plus all remainings
--             inner_builder:add(state.state, uarch_instruction_count - ucycle + 1)
--             ucycle = uarch_instruction_count
--             break
--         end
--     end

--     -- At this point, we've added `uarch_instruction_count - 1` hashes to the inner merkle builder.
--     -- Now we do the last state transition (ureset), and add the last state,
--     -- closing in a power-of-two number of leaves (`2^a` leaves).
--     machine:ureset()
--     local state = machine:state()
--     inner_builder:add(state.state)

--     return inner_builder:build()
-- end
--]]

return CommitmentBuilder
