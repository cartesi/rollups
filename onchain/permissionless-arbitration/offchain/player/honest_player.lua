#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

local Player = require "player.state"
local Client = require "blockchain.client"
local Hash = require "cryptography.hash"

local time = require "utils.time"
local strategy = require "player.strategy"

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
    p:fetch()
    if strategy.react_honestly(p) then break end
    time.sleep(1)
end
