#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

local Player = require "player.honest_strategy"

local time = require "utils.time"

local player_index = tonumber(arg[1])
local tournament = arg[2]
local machine_path = arg[3]
local p
do
    local CommitmentBuilder = require "computation.commitment"
    local builder = CommitmentBuilder:new(machine_path)
    p = Player:new(tournament, player_index, builder, machine_path)
end

while true do
    if p:react() then break end
    time.sleep(1)
end
