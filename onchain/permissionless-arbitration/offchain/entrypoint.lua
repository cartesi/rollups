#!/usr/bin/lua
package.path = package.path .. ";/opt/cartesi/lib/lua/5.4/?.lua"
package.path = package.path .. ";./offchain/?.lua"
package.cpath = package.cpath .. ";/opt/cartesi/lib/lua/5.4/?.so"

local machine_path = "offchain/program/simple-program"
local FF_TIME = 30
local IDLE_LIMIT = 5
local INACTIVE_LIMIT = 10

local helper = require 'utils.helper'
local blockchain_utils = require "blockchain.utils"
local time = require "utils.time"
local blockchain_constants = require "blockchain.constants"
local Blockchain = require "blockchain.node"
local Machine = require "computation.machine"

print "Hello, world!"
os.execute "cd offchain/program && ./gen_machine_simple.sh"

local m = Machine:new_from_path(machine_path)
local initial_hash = m:state().root_hash
local blockchain = Blockchain:new()
time.sleep(2)
local contract = blockchain:deploy_contract(initial_hash)

-- add more player instances here
local cmds = {
    string.format([[sh -c "echo $$ ; exec ./offchain/player/honest_player.lua %d %s %s | tee honest.log"]], 1, contract, machine_path),
    string.format([[sh -c "echo $$ ; exec ./offchain/player/dishonest_player.lua %d %s %s %s | tee dishonest.log"]], 2, contract, machine_path, initial_hash)
}
local pid_reader = {}
local pid_player = {}

for i, cmd in ipairs(cmds) do
    local reader = io.popen(cmd)
    local pid = reader:read()
    pid_reader[pid] = reader
    pid_player[pid] = i
end

-- gracefully end children processes
setmetatable(pid_reader, {
    __gc = function(t)
        helper.stop_players(t)
    end
})

local no_active_players = 0
local all_idle = 0
local last_ts = [[01/01/2000 00:00:00]]
while true do
    local players = 0

    for pid, reader in pairs(pid_reader) do
        local msg_out = 0
        players = players + 1
        last_ts, msg_out = helper.log_to_ts(reader, last_ts)

        -- close the reader and delete the reader entry when there's no more msg in the buffer
        -- and the process has already ended
        if msg_out == 0 and helper.is_zombie(pid) then
            helper.log(pid_player[pid], string.format("player process %s is dead", pid))
            reader:close()
            pid_reader[pid] = nil
            pid_player[pid] = nil
        end
    end

    if players > 0 then
        if helper.all_players_idle(pid_player) then
            all_idle = all_idle + 1
            helper.rm_all_players_idle(pid_player)
        else
            all_idle = 0
        end

        -- if all players are idle for `IDLE_LIMIT` consecutive iterations, advance blockchain
        if all_idle == IDLE_LIMIT then
            print(string.format("all players idle, fastforward blockchain for %d seconds...", FF_TIME))
            blockchain_utils.advance_time(FF_TIME, blockchain_constants.endpoint)
            all_idle = 0
        end
    end

    if players == 0 then
        no_active_players = no_active_players + 1
    else
        no_active_players = 0
    end

    -- if no active player processes for `INACTIVE_LIMIT` consecutive iterations, break loop
    if no_active_players == INACTIVE_LIMIT then
        print("no active players, end program...")
        break
    end
end

print "Good-bye, world!"
