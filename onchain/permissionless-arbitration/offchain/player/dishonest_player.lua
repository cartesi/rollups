#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

local Player = require "player.honest_strategy"
local Client = require "blockchain.client"
local Hash = require "cryptography.hash"

local time = require "utils.time"

local player_index = tonumber(arg[1])
local tournament = arg[2]
local machine_path = arg[3]
local initial_hash = Hash:from_digest_hex(arg[4])
local p
do
    local FakeCommitmentBuilder = require "computation.fake_commitment"
    local builder = FakeCommitmentBuilder:new(initial_hash)
    p = Player:new(tournament, player_index, builder, machine_path)
end

while true do
    if p:react() then break end
    time.sleep(1)
end
