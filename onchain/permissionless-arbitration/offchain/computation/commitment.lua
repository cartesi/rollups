local MerkleBuilder = require "cryptography.merkle_builder"
local Machine = require "computation.machine"

local arithmetic = require "utils.arithmetic"
local consts = require "constants"

local ulte = arithmetic.ulte

local function build_small_machine_commitment(base_cycle, log2_stride_count, machine)
    machine:advance(base_cycle)
    local initial_state = machine:result().state

    local outer_builder = MerkleBuilder:new()
    local big_instructions = 1 << (log2_stride_count - consts.a)
    local big_machine_halted = false

    local big_instruction = 0
    while math.ult(big_instruction, big_instructions) do
        local inner_builder = MerkleBuilder:new()

        -- This loop runs from `1` to `2^a - 1`, for a total of `2^a - 1` times
        local ucycle = 1
        local small_instructions = arithmetic.max_int(consts.a)
        while ulte(ucycle, small_instructions) do
            machine:uadvance(ucycle)
            local state = machine:result()

            if not state.uhalted then
                inner_builder:add(state.state)
            else
                -- add this loop plus all remainings
                inner_builder:add(state.state, small_instructions - ucycle + 1)
                break
            end

            ucycle = ucycle + 1
        end

        -- At this point, we've added `2^a - 1` hashes to the inner merkle builder.
        -- Now we do the last state transition (ureset), and add the last state,
        -- closing in a power-of-two number of leaves (`2^a` leaves).
        machine:ureset()
        local state = machine:result()
        inner_builder:add(state.state)

        if not big_machine_halted then
            outer_builder:add(inner_builder:build())
            big_machine_halted = state.halted
        else
            -- add this loop plus all remainings
            outer_builder:add(inner_builder:build(), big_instructions - big_instruction + 1)
            break
        end

        big_instruction = big_instruction + 1
    end

    return initial_state, outer_builder:build()
end



local function build_big_machine_commitment(base_cycle, log2_stride, log2_stride_count, machine)
    machine:advance(base_cycle + 0)
    local initial_state = machine:result().state

    local builder = MerkleBuilder:new()
    local strides = (1 << log2_stride_count) - 1

    local stride = 0
    while ulte(stride, strides) do
        local cycle = ((stride + 1) << (log2_stride - consts.a))
        machine:advance(base_cycle + cycle)

        local state = machine:result()
        if not state.halted then
            builder:add(state.state)
        else
            -- add this loop plus all remainings
            builder:add(state.state, strides - stride + 1)
            break
        end

        stride = stride + 1
    end

    return initial_state, builder:build()
end

local function build_commitment(base_cycle, log2_stride, log2_stride_count, machine_path)
    local machine = Machine:new_from_path(machine_path)

    if log2_stride >= consts.a then
        assert(log2_stride - consts.a + log2_stride_count <= 63)
        return build_big_machine_commitment(base_cycle, log2_stride, log2_stride_count, machine)
    else
        assert(log2_stride == 0)
        return build_small_machine_commitment(base_cycle, log2_stride_count, machine)
    end
end

local path = "program/simple-program"

local initial, tree = build_commitment(0, 0, 64, path)
-- local initial, tree = build_commitment(0, 64, 63, path)
print(initial, tree.root_hash)

return build_commitment
